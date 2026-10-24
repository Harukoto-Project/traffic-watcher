#[cfg(windows)]
mod imp {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    use anyhow::Context;
    use tokio_util::sync::CancellationToken;
    use windows_service::service::{
        Service, ServiceAccess, ServiceAction, ServiceActionType, ServiceControl,
        ServiceControlAccept, ServiceErrorControl, ServiceExitCode, ServiceFailureActions,
        ServiceFailureResetPeriod, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{
        self, ServiceControlHandlerResult, ServiceStatusHandle,
    };
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    use windows_service::{define_windows_service, service_dispatcher};

    use crate::app;

    pub const SERVICE_NAME: &str = "monitord";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
    const WAIT: Duration = Duration::from_secs(30);

    static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

    define_windows_service!(ffi_service_main, service_main);

    pub fn run_dispatcher(config_path: PathBuf) -> anyhow::Result<()> {
        CONFIG_PATH
            .set(config_path)
            .map_err(|_| anyhow::anyhow!("service dispatcher already started"))?;
        service_dispatcher::start(SERVICE_NAME, ffi_service_main).context(
            "failed to connect to the service control manager (run via the SCM, or use --console)",
        )?;
        Ok(())
    }

    fn service_main(_arguments: Vec<OsString>) {
        if let Err(err) = run_service() {
            tracing::error!(error = %format!("{err:#}"), "service terminated with error");
        }
    }

    fn report(
        handle: &ServiceStatusHandle,
        state: ServiceState,
        accepted: ServiceControlAccept,
        exit_code: ServiceExitCode,
        checkpoint: u32,
        wait_hint: Duration,
    ) -> anyhow::Result<()> {
        handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: state,
            controls_accepted: accepted,
            exit_code,
            checkpoint,
            wait_hint,
            process_id: None,
        })?;
        Ok(())
    }

    fn run_service() -> anyhow::Result<()> {
        let config_path = CONFIG_PATH
            .get()
            .cloned()
            .context("service config path is not set")?;
        let token = CancellationToken::new();
        let handler_token = token.clone();

        let handle = service_control_handler::register(SERVICE_NAME, move |event| match event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                handler_token.cancel();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        })?;

        report(
            &handle,
            ServiceState::StartPending,
            ServiceControlAccept::empty(),
            ServiceExitCode::Win32(0),
            1,
            Duration::from_secs(10),
        )?;

        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(err) => {
                let _ = report(
                    &handle,
                    ServiceState::Stopped,
                    ServiceControlAccept::empty(),
                    ServiceExitCode::ServiceSpecific(2),
                    0,
                    Duration::default(),
                );
                return Err(err.into());
            }
        };

        report(
            &handle,
            ServiceState::Running,
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            ServiceExitCode::Win32(0),
            0,
            Duration::default(),
        )?;

        let result = runtime.block_on(app::run(config_path, token));
        drop(runtime);

        if let Err(err) = &result {
            tracing::error!(error = %format!("{err:#}"), "monitord exited with error");
        }
        let exit_code = if result.is_ok() {
            ServiceExitCode::Win32(0)
        } else {
            ServiceExitCode::ServiceSpecific(1)
        };
        report(
            &handle,
            ServiceState::Stopped,
            ServiceControlAccept::empty(),
            exit_code,
            0,
            Duration::default(),
        )?;
        result
    }

    fn open(access: ServiceAccess) -> anyhow::Result<Service> {
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
            .context("failed to open the service control manager")?;
        manager.open_service(SERVICE_NAME, access).with_context(|| {
            format!(
                "failed to open service `{SERVICE_NAME}` (is it installed? admin rights required)"
            )
        })
    }

    pub fn install(config_path: &Path) -> anyhow::Result<()> {
        let executable_path =
            std::env::current_exe().context("cannot determine executable path")?;
        let manager = ServiceManager::local_computer(
            None::<&str>,
            ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
        )
        .context("failed to open the service control manager (admin rights required)")?;

        let info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from("Traffic Watcher Monitor"),
            service_type: SERVICE_TYPE,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path,
            launch_arguments: vec![
                OsString::from("--config"),
                config_path.as_os_str().to_owned(),
                OsString::from("service"),
            ],
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };

        let service = manager
            .create_service(
                &info,
                ServiceAccess::CHANGE_CONFIG | ServiceAccess::QUERY_STATUS | ServiceAccess::START,
            )
            .context("failed to create the service (already installed?)")?;
        service.set_description(
            "Monitors network latency and system load, records metrics and raises alerts.",
        )?;
        service.update_failure_actions(ServiceFailureActions {
            reset_period: ServiceFailureResetPeriod::After(Duration::from_secs(86_400)),
            reboot_msg: None,
            command: None,
            actions: Some(vec![
                ServiceAction {
                    action_type: ServiceActionType::Restart,
                    delay: Duration::from_secs(5),
                },
                ServiceAction {
                    action_type: ServiceActionType::Restart,
                    delay: Duration::from_secs(30),
                },
                ServiceAction {
                    action_type: ServiceActionType::Restart,
                    delay: Duration::from_secs(60),
                },
            ]),
        })?;
        service.set_failure_actions_on_non_crash_failures(true)?;
        Ok(())
    }

    fn wait_for(service: &Service, target: ServiceState) -> anyhow::Result<()> {
        let started = Instant::now();
        loop {
            let status = service.query_status()?;
            if status.current_state == target {
                return Ok(());
            }
            if target == ServiceState::Running && status.current_state == ServiceState::Stopped {
                anyhow::bail!(
                    "the service stopped right after starting (see the log file in data_dir\\logs)"
                );
            }
            if started.elapsed() > WAIT {
                anyhow::bail!("timed out waiting for the service to reach {target:?}");
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    }

    pub fn uninstall() -> anyhow::Result<()> {
        let service =
            open(ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE)?;
        if service.query_status()?.current_state != ServiceState::Stopped {
            let _ = service.stop();
            wait_for(&service, ServiceState::Stopped)?;
        }
        service.delete().context("failed to delete the service")?;
        Ok(())
    }

    pub fn start() -> anyhow::Result<()> {
        let service = open(ServiceAccess::QUERY_STATUS | ServiceAccess::START)?;
        if service.query_status()?.current_state == ServiceState::Running {
            return Ok(());
        }
        service
            .start::<&str>(&[])
            .context("failed to start the service")?;
        wait_for(&service, ServiceState::Running)
    }

    pub fn stop() -> anyhow::Result<()> {
        let service = open(ServiceAccess::QUERY_STATUS | ServiceAccess::STOP)?;
        if service.query_status()?.current_state == ServiceState::Stopped {
            return Ok(());
        }
        service.stop().context("failed to stop the service")?;
        wait_for(&service, ServiceState::Stopped)
    }
}

#[cfg(not(windows))]
mod imp {
    use std::path::{Path, PathBuf};

    pub fn run_dispatcher(_config_path: PathBuf) -> anyhow::Result<()> {
        anyhow::bail!("Windows services are only supported on Windows")
    }

    pub fn install(_config_path: &Path) -> anyhow::Result<()> {
        anyhow::bail!("Windows services are only supported on Windows")
    }

    pub fn uninstall() -> anyhow::Result<()> {
        anyhow::bail!("Windows services are only supported on Windows")
    }

    pub fn start() -> anyhow::Result<()> {
        anyhow::bail!("Windows services are only supported on Windows")
    }

    pub fn stop() -> anyhow::Result<()> {
        anyhow::bail!("Windows services are only supported on Windows")
    }
}

pub use imp::{install, run_dispatcher, start, stop, uninstall};
