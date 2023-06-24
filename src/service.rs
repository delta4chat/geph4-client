use std::{error::Error, ffi::OsString, time::Duration, sync::mpsc};
use windows_service::{
    service::{
        ServiceAccess, ServiceControl, ServiceErrorControl, ServiceInfo, ServiceStartType,
        ServiceType, ServiceStatus, ServiceState, ServiceControlAccept, ServiceExitCode,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess}, define_windows_service,
};
use crate::dispatch;

const SERVICE_NAME: &str = "Geph";
const SERVICE_DISPLAY_NAME: &str = "Geph";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(args: Vec<OsString>) -> anyhow::Result<()> {
    if let Err(e) = run_service(args) {
        eprintln!("Error running service: {}", e);
    }
    Ok(())
}

fn run_service(args: Vec<OsString>) -> windows_service::Result<()> {
    eprintln!("Running service");
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                shutdown_tx.send(()).expect("Unable to shutdown service");
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    match dispatch() {
        Ok(_) => (),
        Err(e) => eprintln!("Error dispatching client: {}", e),
    };

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

pub fn start() -> windows_service::Result<()> {
    match service_dispatcher::start(SERVICE_NAME, ffi_service_main) {
        Ok(_) => (),
        Err(e) => println!("error: {:?}", e.source()),
    };

    Ok(())
}

pub fn install() -> windows_service::Result<()> {
    eprintln!("Intitiating service install");
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access);
    match &service_manager {
        Ok(_) => (),
        Err(e) => println!("Error instantiating service manager: {:?}", e.source()),
    }

    let service_binary_path = std::env::current_exe()
        .expect("Error retreiving service path")
        .with_file_name("geph4-client.exe");
    println!("binary path: {:?}", service_binary_path);

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };
    let service = service_manager?.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(
        "Geph connects you with the censorship-free Internet, even when nothing else works.",
    )?;

    Ok(())
}
