use std::ffi::c_void;
use std::io;
use std::time::Duration;

use anyhow::Context;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows::core::PCWSTR;

use crate::protocol::{Envelope, Request, Response};

const SDDL: &str = "D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;IU)";
const MAX_REQUEST_BYTES: u64 = 64 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const ERROR_PIPE_BUSY: i32 = 231;

struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

unsafe impl Send for SecurityDescriptor {}
unsafe impl Sync for SecurityDescriptor {}

impl SecurityDescriptor {
    fn from_sddl(sddl: &str) -> anyhow::Result<Self> {
        let wide: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(wide.as_ptr()),
                SDDL_REVISION_1,
                &mut descriptor,
                None,
            )
        }
        .context("failed to build pipe security descriptor")?;
        Ok(Self(descriptor))
    }

    fn create_server(&self, addr: &str, first: bool) -> io::Result<NamedPipeServer> {
        let mut attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0.0,
            bInheritHandle: false.into(),
        };
        let mut options = ServerOptions::new();
        options
            .first_pipe_instance(first)
            .reject_remote_clients(true);
        unsafe {
            options.create_with_security_attributes_raw(
                addr,
                &mut attributes as *mut SECURITY_ATTRIBUTES as *mut c_void,
            )
        }
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            unsafe {
                let _ = LocalFree(Some(HLOCAL(self.0.0)));
            }
        }
    }
}

fn pipe_address(name: &str) -> String {
    format!(r"\\.\pipe\{name}")
}

pub async fn serve(
    pipe_name: &str,
    tx: Sender<Envelope>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let addr = pipe_address(pipe_name);
    let descriptor = SecurityDescriptor::from_sddl(SDDL)?;
    let mut server = descriptor
        .create_server(&addr, true)
        .with_context(|| format!("failed to create named pipe {addr}"))?;
    tracing::info!(pipe = %addr, "ipc server listening");

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => return Ok(()),
            connected = server.connect() => {
                connected.context("named pipe connect failed")?;
            }
        }
        let client = server;
        server = descriptor
            .create_server(&addr, false)
            .context("failed to create next named pipe instance")?;
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_client(client, tx).await {
                tracing::debug!(%err, "ipc client ended with error");
            }
        });
    }
}

async fn handle_client(pipe: NamedPipeServer, tx: Sender<Envelope>) -> anyhow::Result<()> {
    let (reader, mut writer) = tokio::io::split(pipe);
    let mut reader = BufReader::new(reader);

    loop {
        let mut line = String::new();
        let read = {
            let mut limited = (&mut reader).take(MAX_REQUEST_BYTES);
            tokio::time::timeout(READ_TIMEOUT, limited.read_line(&mut line)).await
        };
        let n = match read {
            Ok(result) => result?,
            Err(_) => return Ok(()),
        };
        if n == 0 {
            return Ok(());
        }
        if !line.ends_with('\n') {
            write_response(&mut writer, &Response::err("request too large")).await?;
            return Ok(());
        }

        let response = match serde_json::from_str::<Request>(line.trim()) {
            Ok(request) => dispatch(&tx, request).await,
            Err(err) => Response::err(format!("invalid request: {err}")),
        };
        write_response(&mut writer, &response).await?;
    }
}

async fn dispatch(tx: &Sender<Envelope>, request: Request) -> Response {
    let (reply, rx) = oneshot::channel();
    if tx.send(Envelope { request, reply }).await.is_err() {
        return Response::err("daemon is shutting down");
    }
    match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(response)) => response,
        Ok(Err(_)) => Response::err("request was dropped"),
        Err(_) => Response::err("request timed out"),
    }
}

async fn write_response<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    response: &Response,
) -> anyhow::Result<()> {
    let mut bytes = serde_json::to_vec(response)?;
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn request(pipe_name: &str, request: &Request) -> anyhow::Result<Response> {
    let addr = pipe_address(pipe_name);
    let mut attempts = 0;
    let client = loop {
        match ClientOptions::new().open(&addr) {
            Ok(client) => break client,
            Err(err) if err.raw_os_error() == Some(ERROR_PIPE_BUSY) && attempts < 40 => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                anyhow::bail!("monitord is not running (pipe {addr} not found)")
            }
            Err(err) => return Err(err).with_context(|| format!("failed to open pipe {addr}")),
        }
    };

    let (reader, mut writer) = tokio::io::split(client);
    let mut payload = serde_json::to_vec(request)?;
    payload.push(b'\n');
    writer.write_all(&payload).await?;
    writer.flush().await?;

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let n = tokio::time::timeout(Duration::from_secs(60), reader.read_line(&mut line))
        .await
        .context("timed out waiting for response")??;
    anyhow::ensure!(n > 0, "connection closed without response");
    Ok(serde_json::from_str(line.trim())?)
}
