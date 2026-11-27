use super::{
    NativeServices, ServiceError,
    health::{HEALTH_TIMEOUT, health_ready_at},
    registry::{ManagedProcess, SERVICE_DEFINITIONS, ServiceKey},
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    sync::Arc,
    thread,
};

#[test]
fn service_error_serialization_keeps_category() {
    let missing = ServiceError::ExecutableMissing("missing binary".into());
    assert_eq!(
        serde_json::to_value(missing).expect("serialize"),
        serde_json::json!({"kind": "executable_missing", "details": "missing binary"})
    );
    assert_eq!(
        serde_json::to_value(ServiceError::RegistryUnavailable).expect("serialize"),
        serde_json::json!({"kind": "registry_unavailable"})
    );
}

#[test]
fn known_service_names_and_loopback_addresses() {
    for kind in SERVICE_DEFINITIONS
        .iter()
        .map(|definition| ServiceKey(definition.name))
    {
        assert_eq!(ServiceKey::parse(kind.name()), Ok(kind));
        assert!(kind.address().ip().is_loopback());
    }
    assert!(ServiceKey::parse("../server").is_err());
}

#[test]
fn empty_registry_rejects_unowned_services() {
    let services = NativeServices::default();
    assert!(services.stop_blocking("ai").is_err());
    assert!(services.stop_blocking("image").is_err());
    assert!(services.stop_blocking("speech").is_err());
}

#[test]
fn clones_share_one_registry_owner() {
    let services = NativeServices::default();
    let clone = services.clone();
    assert!(Arc::ptr_eq(&services.0, &clone.0));
    assert_eq!(clone.status_blocking("speech").unwrap().service, "speech");
}

#[test]
fn occupied_external_port_is_not_owned_or_stopped() {
    let address = ServiceKey::parse("image").unwrap().address();
    let Ok(listener) = TcpListener::bind(address) else {
        return;
    };
    let services = NativeServices::default();
    assert!(services.start_blocking("image").is_err());
    assert!(services.stop_blocking("image").is_err());
    assert!(TcpStream::connect_timeout(&address, HEALTH_TIMEOUT).is_ok());
    drop(listener);
}

#[test]
fn non_health_http_response_is_not_readiness() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
    let address = listener.local_addr().expect("listener address");
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        let mut request = [0; 256];
        let _ = stream.read(&mut request);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 17\r\n\r\n{\"status\":\"bad\"}")
            .expect("write response");
    });
    assert!(!health_ready_at(
        ServiceKey::parse("image").unwrap(),
        address
    ));
    worker.join().expect("join worker");
}

#[test]
fn speech_plaintext_health_is_accepted() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
    let address = listener.local_addr().expect("listener address");
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        let mut request = [0; 256];
        let _ = stream.read(&mut request);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
            .expect("respond");
    });
    assert!(health_ready_at(
        ServiceKey::parse("speech").unwrap(),
        address
    ));
    worker.join().expect("join worker");
}

#[test]
fn shutdown_reaps_only_owned_children() {
    #[cfg(windows)]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "ping -n 30 127.0.0.1 >NUL"]);
        command
    };
    #[cfg(not(windows))]
    let mut command = {
        let mut command = Command::new("sleep");
        command.arg("30");
        command
    };
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn owned test child");
    let services = NativeServices::default();
    services.lock().expect("registry").processes.insert(
        ServiceKey::parse("image").unwrap(),
        ManagedProcess { child },
    );
    services.shutdown();
    assert!(services.lock().expect("registry").processes.is_empty());
}

#[test]
fn valid_health_response_is_readiness() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
    let address = listener.local_addr().expect("listener address");
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        let mut request = [0; 256];
        let _ = stream.read(&mut request);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\n\r\n{\"status\":\"ok\"}")
            .expect("write response");
    });
    assert!(health_ready_at(
        ServiceKey::parse("image").unwrap(),
        address
    ));
    worker.join().expect("join worker");
}
