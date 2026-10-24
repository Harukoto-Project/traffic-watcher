mod protocol;
pub mod query;

#[cfg(windows)]
mod pipe;

#[cfg(not(windows))]
mod pipe {
    use tokio::sync::mpsc::Sender;
    use tokio_util::sync::CancellationToken;

    use crate::protocol::{Envelope, Request, Response};

    pub async fn serve(
        _pipe_name: &str,
        _tx: Sender<Envelope>,
        _shutdown: CancellationToken,
    ) -> anyhow::Result<()> {
        anyhow::bail!("named pipe IPC is only supported on Windows")
    }

    pub async fn request(_pipe_name: &str, _request: &Request) -> anyhow::Result<Response> {
        anyhow::bail!("named pipe IPC is only supported on Windows")
    }
}

pub use pipe::{request, serve};
pub use protocol::{DEFAULT_PIPE_NAME, Envelope, Request, Response};
