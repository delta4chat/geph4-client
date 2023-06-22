use std::ffi::OsString;
use windows_service::service::ServiceControl;
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::{service_dispatcher, Result};

const SERVICE_NAME: &str = "Geph";

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(args: Vec<OsString>) -> anyhow::Result<()> {
    if let Err(e) = run_service(args) {}
}

fn run_service(arguments: Vec<OsString>) -> Result<()> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => ServiceControlHandlerResult::NoError,
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    let next_status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };

    // Tell the system that the service is running now
    status_handle.set_service_status(next_status)?;

    Ok(())
}

pub fn start() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;

    Ok(())
}
