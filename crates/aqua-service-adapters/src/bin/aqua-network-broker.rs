#[cfg(all(feature = "wifi-native", target_os = "linux"))]
use aqua_service_adapters::network_broker::FIXED_WIFI_INTERFACE;
use aqua_service_adapters::network_broker::{
    parse_authenticated_request, parse_supervisor_state, AuthenticatedNetworkRequest,
    NetworkBrokerOperation, NetworkBrokerRequest, NetworkSupervisorState, WifiBrokerRequest,
    FIXED_INTERFACE, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, MAX_STATE_BYTES, PROTOCOL_VERSION,
};
#[cfg(all(feature = "wifi-native", target_os = "linux"))]
use aqua_service_adapters::wifi_control::{
    validate_credential_metadata, WifiControlRequest, WifiControlResponse, WifiControlStatus,
    WifiCredentialMetadata, WifiCredentialRecord, WifiSecurity, WifiSsid,
    MAX_WIFI_CREDENTIAL_RECORD_BYTES, WIFI_CREDENTIAL_DIRECTORY, WIFI_CREDENTIAL_DIRECTORY_MODE,
    WIFI_CREDENTIAL_FILE_MODE, WIFI_CREDENTIAL_PATH, WIFI_CREDENTIAL_TEMP_PATH,
};
#[cfg(all(feature = "wifi-native", target_os = "linux"))]
use aqua_service_adapters::{derive_wpa2_psk, WifiNativeControl, WifiNativeError};
use std::env;
use std::ffi::CString;
use std::fs;
#[cfg(all(feature = "wifi-native", target_os = "linux"))]
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
#[cfg(all(feature = "wifi-native", target_os = "linux"))]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::ExitCode;
use std::thread;
#[cfg(all(feature = "wifi-native", target_os = "linux"))]
use std::time::Instant;
use std::time::{Duration, SystemTime};

const SOCKET_PATH: &str = aqua_service_adapters::network_broker::NETWORK_BROKER_SOCKET_PATH;
const STATE_PATH: &str = "/run/aqua-network/network-service-supervisor.state";
const READY_PATH: &str = "/run/aqua-network/lease.ready";
#[cfg(all(feature = "wifi-native", target_os = "linux"))]
const WIFI_ASSOCIATION_PATH: &str = "/run/aqua-network/wifi.associated";
const AQUA_UID: u32 = 1000;
const AQUA_GID: u32 = 1000;
const IO_TIMEOUT: Duration = Duration::from_secs(2);
const RENEW_TIMEOUT: Duration = Duration::from_secs(8);
#[cfg(all(feature = "wifi-native", target_os = "linux"))]
const BROKER_OPERATIONS: &str =
    "status,renew-dhcp,wifi-status,wifi-connect,wifi-reconnect,wifi-disconnect";
#[cfg(not(all(feature = "wifi-native", target_os = "linux")))]
const BROKER_OPERATIONS: &str = "status,renew-dhcp";

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    let result = match args.as_slice() {
        [_, command] if command == "serve" => serve(),
        [_, command] if command == "status" => request(NetworkBrokerOperation::Status),
        [_, command] if command == "renew-dhcp" => request(NetworkBrokerOperation::RenewDhcp),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: aqua-network-broker serve|status|renew-dhcp",
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!(
                "[AQUA-NETWORK] stage=network-privilege-broker status=failed reason={}",
                error.kind()
            );
            ExitCode::FAILURE
        }
    }
}

fn serve() -> io::Result<()> {
    if unsafe { libc::geteuid() } != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "root broker required",
        ));
    }
    let socket_path = Path::new(SOCKET_PATH);
    validate_control_directory(socket_path.parent().expect("fixed socket parent"))?;
    remove_stale_socket(socket_path)?;
    let listener = UnixListener::bind(socket_path)?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o660))?;
    chown_socket(socket_path, 0, AQUA_GID)?;
    println!(
        "[AQUA-NETWORK] stage=network-privilege-broker status=ready socket={} owner_uid=0 client_uid={} interface={} operations={} arbitrary_commands=false arbitrary_paths=false",
        SOCKET_PATH, AQUA_UID, FIXED_INTERFACE, BROKER_OPERATIONS
    );

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream) {
                    eprintln!(
                        "[AQUA-NETWORK] stage=network-privilege-broker status=request-failed reason={}",
                        error.kind()
                    );
                }
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: UnixStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    let peer = peer_credentials(&stream)?;
    if peer.uid != AQUA_UID || peer.gid != AQUA_GID {
        let mut denied_request = Vec::with_capacity(MAX_REQUEST_BYTES + 1);
        Read::by_ref(&mut stream)
            .take((MAX_REQUEST_BYTES + 1) as u64)
            .read_to_end(&mut denied_request)?;
        write_response(
            &mut stream,
            &format!("{PROTOCOL_VERSION} ERROR unauthorized-peer\n"),
        )?;
        eprintln!(
            "[AQUA-NETWORK] stage=network-privilege-broker status=rejected reason=unauthorized-peer peer_uid={} peer_gid={} peer_pid={}",
            peer.uid, peer.gid, peer.pid
        );
        return Ok(());
    }

    let mut bytes = Vec::with_capacity(MAX_REQUEST_BYTES + 1);
    Read::by_ref(&mut stream)
        .take((MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let parsed_request = parse_authenticated_request(&bytes);
    wipe_bytes(&mut bytes);
    let request = match parsed_request {
        Ok(request) => request,
        Err(error) => {
            write_response(&mut stream, &format!("{PROTOCOL_VERSION} ERROR {error}\n"))?;
            return Ok(());
        }
    };
    match request {
        AuthenticatedNetworkRequest::Ethernet(request) => {
            let state = read_state(Path::new(STATE_PATH))?;
            match request.operation {
                NetworkBrokerOperation::Status => {
                    write_response(&mut stream, &state.status_response())?
                }
                NetworkBrokerOperation::RenewDhcp => renew_dhcp(&mut stream, &state)?,
            }
        }
        AuthenticatedNetworkRequest::Wifi(request) => handle_wifi_request(&mut stream, request)?,
    }
    Ok(())
}

fn wipe_bytes(bytes: &mut [u8]) {
    for byte in bytes {
        unsafe { std::ptr::write_volatile(byte, 0) };
    }
}

#[cfg(not(all(feature = "wifi-native", target_os = "linux")))]
fn handle_wifi_request(stream: &mut UnixStream, _request: WifiBrokerRequest) -> io::Result<()> {
    write_response(
        stream,
        &format!("{PROTOCOL_VERSION} ERROR wifi-native-unavailable\n"),
    )
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn handle_wifi_request(stream: &mut UnixStream, request: WifiBrokerRequest) -> io::Result<()> {
    let result = match request {
        WifiBrokerRequest::Status => wifi_status(),
        WifiBrokerRequest::Connect { ssid, passphrase } => wifi_connect(ssid, passphrase),
        WifiBrokerRequest::Reconnect => wifi_reconnect(),
        WifiBrokerRequest::Disconnect => wifi_disconnect(),
    };
    match result {
        Ok(response) => write_response(stream, &response),
        Err(reason) => write_response(stream, &format!("{PROTOCOL_VERSION} ERROR {reason}\n")),
    }
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn wifi_status() -> Result<String, &'static str> {
    let mut control = WifiNativeControl::connect().map_err(native_reason)?;
    let response = control
        .request(&WifiControlRequest::Status)
        .map_err(native_reason)?;
    let WifiControlResponse::Status(status) = response else {
        return Err("invalid-wifi-status");
    };
    Ok(format!(
        "{PROTOCOL_VERSION} OK operation=wifi-status interface={FIXED_WIFI_INTERFACE} state={} network_id={} authoritative={}\n",
        status.state.id(),
        status.network_id.map_or_else(|| "none".to_owned(), |id| id.to_string()),
        status.authoritative_association()
    ))
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn wifi_disconnect() -> Result<String, &'static str> {
    let mut control = WifiNativeControl::connect().map_err(native_reason)?;
    let network_id = match control
        .request(&WifiControlRequest::Status)
        .map_err(native_reason)?
    {
        WifiControlResponse::Status(status) => status.network_id,
        _ => return Err("invalid-wifi-status"),
    };
    control
        .request(&WifiControlRequest::Disconnect)
        .map_err(native_reason)?;
    if let Some(network_id) = network_id {
        expect_acknowledgement(control.request(&WifiControlRequest::RemoveNetwork { network_id }))
            .map_err(native_reason)?;
    }
    remove_wifi_association_marker().map_err(|_| "association-state-write-failed")?;
    Ok(format!(
        "{PROTOCOL_VERSION} OK operation=wifi-disconnect interface={FIXED_WIFI_INTERFACE} authoritative=true\n"
    ))
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn wifi_connect(
    ssid: WifiSsid,
    passphrase: aqua_service_adapters::wifi_control::WifiPassphrase,
) -> Result<String, &'static str> {
    remove_wifi_association_marker().map_err(|_| "association-state-write-failed")?;
    let psk = derive_wpa2_psk(&ssid, &passphrase).map_err(native_reason)?;
    drop(passphrase);
    let status = wifi_associate(&ssid, &psk)?;
    let record = WifiCredentialRecord::new(ssid, WifiSecurity::Wpa2Personal, psk);
    persist_wifi_credential(&record).map_err(|_| "credential-write-failed")?;
    Ok(format!(
        "{PROTOCOL_VERSION} OK operation=wifi-connect interface={FIXED_WIFI_INTERFACE} network_id={} authoritative={} credential_saved=true\n",
        status.network_id.expect("authoritative status includes network id"),
        status.authoritative_association()
    ))
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn wifi_reconnect() -> Result<String, &'static str> {
    remove_wifi_association_marker().map_err(|_| "association-state-write-failed")?;
    let record = load_wifi_credential().map_err(|_| "credential-read-failed")?;
    let status = wifi_associate(record.ssid(), record.psk())?;
    Ok(format!(
        "{PROTOCOL_VERSION} OK operation=wifi-reconnect interface={FIXED_WIFI_INTERFACE} network_id={} authoritative={} credential_saved=true\n",
        status.network_id.expect("authoritative status includes network id"),
        status.authoritative_association()
    ))
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn wifi_associate(
    ssid: &WifiSsid,
    psk: &aqua_service_adapters::wifi_control::WifiPsk,
) -> Result<WifiControlStatus, &'static str> {
    let mut control = WifiNativeControl::connect().map_err(native_reason)?;
    let network_id = match control
        .request(&WifiControlRequest::AddNetwork)
        .map_err(native_reason)?
    {
        WifiControlResponse::NetworkAdded(network_id) => network_id,
        _ => return Err("invalid-network-id"),
    };
    let association = (|| {
        expect_acknowledgement(control.request(&WifiControlRequest::SetSsid { network_id, ssid }))?;
        expect_acknowledgement(
            control.request(&WifiControlRequest::SetWpa2Personal { network_id }),
        )?;
        expect_acknowledgement(control.request(&WifiControlRequest::SetPsk { network_id, psk }))?;
        expect_acknowledgement(control.request(&WifiControlRequest::EnableNetwork { network_id }))?;
        expect_acknowledgement(control.request(&WifiControlRequest::SelectNetwork { network_id }))?;
        wait_for_association(&mut control, network_id)
    })();
    let status = match association {
        Ok(status) => status,
        Err(error) => {
            let _ = control.request(&WifiControlRequest::RemoveNetwork { network_id });
            return Err(native_reason(error));
        }
    };
    persist_wifi_association_marker(
        status
            .network_id
            .expect("authoritative status includes network id"),
    )
    .map_err(|_| "association-state-write-failed")?;
    Ok(status)
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn persist_wifi_association_marker(network_id: u16) -> io::Result<()> {
    let path = Path::new(WIFI_ASSOCIATION_PATH);
    let directory = path.parent().expect("fixed association marker parent");
    validate_control_directory(directory)?;
    let temporary = directory.join(format!(".wifi.associated.{}", std::process::id()));
    let mut transaction = CredentialTransaction::new(&temporary);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o644)
        .open(&temporary)?;
    writeln!(file, "product=Aqua Linux")?;
    writeln!(file, "interface={FIXED_WIFI_INTERFACE}")?;
    writeln!(file, "network_id={network_id}")?;
    writeln!(file, "authoritative=true")?;
    fs::set_permissions(&temporary, fs::Permissions::from_mode(0o644))?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    fs::File::open(directory)?.sync_all()?;
    transaction.commit();
    Ok(())
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn remove_wifi_association_marker() -> io::Result<()> {
    match fs::symlink_metadata(WIFI_ASSOCIATION_PATH) {
        Ok(metadata) if metadata.file_type().is_file() && !metadata.file_type().is_symlink() => {
            fs::remove_file(WIFI_ASSOCIATION_PATH)
        }
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "unsafe Wi-Fi association marker",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn expect_acknowledgement(
    response: Result<WifiControlResponse, WifiNativeError>,
) -> Result<(), WifiNativeError> {
    match response? {
        WifiControlResponse::Acknowledged => Ok(()),
        _ => Err(WifiNativeError::ApiFailed),
    }
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn wait_for_association(
    control: &mut WifiNativeControl,
    network_id: u16,
) -> Result<WifiControlStatus, WifiNativeError> {
    let started = Instant::now();
    let timeout = Duration::from_secs(15);
    while started.elapsed() < timeout {
        let response = control.request(&WifiControlRequest::Status)?;
        if let WifiControlResponse::Status(status) = response {
            if status.authoritative_association() && status.network_id == Some(network_id) {
                return Ok(status);
            }
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err(WifiNativeError::Timeout)
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn persist_wifi_credential(record: &WifiCredentialRecord) -> io::Result<()> {
    let directory = Path::new(WIFI_CREDENTIAL_DIRECTORY);
    match fs::symlink_metadata(directory) {
        Ok(metadata) => validate_credential_directory(&metadata)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::DirBuilder::new()
                .mode(WIFI_CREDENTIAL_DIRECTORY_MODE)
                .create(directory)?;
            validate_credential_directory(&fs::symlink_metadata(directory)?)?;
        }
        Err(error) => return Err(error),
    }

    let temporary = Path::new(WIFI_CREDENTIAL_TEMP_PATH);
    if fs::symlink_metadata(temporary).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "stale Wi-Fi credential transaction",
        ));
    }
    let payload = record.encode();
    if payload.bytes().is_empty() || payload.bytes().len() > MAX_WIFI_CREDENTIAL_RECORD_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid Wi-Fi credential payload",
        ));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(WIFI_CREDENTIAL_FILE_MODE)
        .open(temporary)?;
    let mut transaction = CredentialTransaction::new(temporary);
    file.write_all(payload.bytes())?;
    file.sync_all()?;
    let directory_metadata = fs::symlink_metadata(directory)?;
    let file_metadata = fs::symlink_metadata(temporary)?;
    validate_credential_metadata(WifiCredentialMetadata {
        directory_uid: directory_metadata.uid(),
        directory_mode: directory_metadata.mode() & 0o777,
        directory_is_symlink: directory_metadata.file_type().is_symlink(),
        file_uid: file_metadata.uid(),
        file_mode: file_metadata.mode() & 0o777,
        file_is_regular: file_metadata.file_type().is_file(),
        file_is_symlink: file_metadata.file_type().is_symlink(),
        file_bytes: usize::try_from(file_metadata.len()).unwrap_or(usize::MAX),
    })
    .map_err(|_| io::Error::new(io::ErrorKind::PermissionDenied, "unsafe credential storage"))?;
    let destination = Path::new(WIFI_CREDENTIAL_PATH);
    if let Ok(metadata) = fs::symlink_metadata(destination) {
        if !metadata.file_type().is_file()
            || metadata.file_type().is_symlink()
            || metadata.uid() != 0
            || metadata.mode() & 0o777 != WIFI_CREDENTIAL_FILE_MODE
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "unsafe existing credential",
            ));
        }
    }
    fs::rename(temporary, destination)?;
    fs::File::open(directory)?.sync_all()?;
    transaction.commit();
    Ok(())
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn load_wifi_credential() -> io::Result<WifiCredentialRecord> {
    let directory = Path::new(WIFI_CREDENTIAL_DIRECTORY);
    let path = Path::new(WIFI_CREDENTIAL_PATH);
    let directory_metadata = fs::symlink_metadata(directory)?;
    validate_credential_directory(&directory_metadata)?;
    let metadata = fs::symlink_metadata(path)?;
    validate_credential_metadata(WifiCredentialMetadata {
        directory_uid: directory_metadata.uid(),
        directory_mode: directory_metadata.mode() & 0o777,
        directory_is_symlink: directory_metadata.file_type().is_symlink(),
        file_uid: metadata.uid(),
        file_mode: metadata.mode() & 0o777,
        file_is_regular: metadata.file_type().is_file(),
        file_is_symlink: metadata.file_type().is_symlink(),
        file_bytes: usize::try_from(metadata.len()).unwrap_or(usize::MAX),
    })
    .map_err(|_| io::Error::new(io::ErrorKind::PermissionDenied, "unsafe credential storage"))?;
    let mut bytes = fs::read(path)?;
    let record = WifiCredentialRecord::parse(&bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid Wi-Fi credential"));
    wipe_bytes(&mut bytes);
    record
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
struct CredentialTransaction<'a> {
    path: &'a Path,
    committed: bool,
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
impl<'a> CredentialTransaction<'a> {
    const fn new(path: &'a Path) -> Self {
        Self {
            path,
            committed: false,
        }
    }

    fn commit(&mut self) {
        self.committed = true;
    }
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
impl Drop for CredentialTransaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(self.path);
        }
    }
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn validate_credential_directory(metadata: &fs::Metadata) -> io::Result<()> {
    if !metadata.file_type().is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != 0
        || metadata.mode() & 0o777 != WIFI_CREDENTIAL_DIRECTORY_MODE
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "unsafe Wi-Fi credential directory",
        ));
    }
    Ok(())
}

#[cfg(all(feature = "wifi-native", target_os = "linux"))]
fn native_reason(error: WifiNativeError) -> &'static str {
    match error {
        WifiNativeError::ConnectFailed => "wifi-control-unavailable",
        WifiNativeError::Timeout => "wifi-control-timeout",
        WifiNativeError::Control(_) => "wifi-control-rejected",
        WifiNativeError::DerivationFailed => "psk-derivation-failed",
        _ => "wifi-control-failed",
    }
}

fn renew_dhcp(stream: &mut UnixStream, state: &NetworkSupervisorState) -> io::Result<()> {
    if !state.authoritative_ready() {
        return write_response(
            stream,
            &format!("{PROTOCOL_VERSION} ERROR network-not-ready\n"),
        );
    }
    let pid = state.client_pid.expect("authoritative state has a PID");
    let before = fs::metadata(READY_PATH).and_then(|metadata| metadata.modified())?;
    let signal_result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGUSR1) };
    if signal_result != 0 {
        return write_response(
            stream,
            &format!("{PROTOCOL_VERSION} ERROR renew-signal-failed\n"),
        );
    }

    let started = SystemTime::now();
    while started.elapsed().unwrap_or(RENEW_TIMEOUT) < RENEW_TIMEOUT {
        if renewal_is_authoritative(before)? {
            return write_response(
                stream,
                &format!(
                    "{PROTOCOL_VERSION} OK operation=renew-dhcp interface={FIXED_INTERFACE} authoritative=true\n"
                ),
            );
        }
        thread::sleep(Duration::from_millis(100));
    }
    write_response(stream, &format!("{PROTOCOL_VERSION} ERROR renew-timeout\n"))
}

fn renewal_is_authoritative(before: SystemTime) -> io::Result<bool> {
    let metadata = fs::metadata(READY_PATH)?;
    if metadata.modified()? <= before || metadata.len() > MAX_STATE_BYTES as u64 {
        return Ok(false);
    }
    let content = fs::read(READY_PATH)?;
    Ok(content
        .windows(b"event=renew\n".len())
        .any(|window| window == b"event=renew\n"))
}

fn write_response(stream: &mut UnixStream, response: &str) -> io::Result<()> {
    if response.len() > MAX_RESPONSE_BYTES || !response.ends_with('\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid broker response",
        ));
    }
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn request(operation: NetworkBrokerOperation) -> io::Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    stream.set_read_timeout(Some(IO_TIMEOUT + RENEW_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    let request = NetworkBrokerRequest { operation }.encode();
    stream.write_all(request.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = Vec::with_capacity(MAX_RESPONSE_BYTES + 1);
    stream
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut response)?;
    if response.len() > MAX_RESPONSE_BYTES || !response.ends_with(b"\n") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid broker response",
        ));
    }
    let response = std::str::from_utf8(&response)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid broker response"))?;
    print!("{response}");
    if !response.starts_with(&format!("{PROTOCOL_VERSION} OK ")) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "broker rejected request",
        ));
    }
    Ok(())
}

fn read_state(path: &Path) -> io::Result<NetworkSupervisorState> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_STATE_BYTES as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsafe supervisor state",
        ));
    }
    let bytes = fs::read(path)?;
    parse_supervisor_state(&bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid supervisor state"))
}

fn validate_control_directory(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_dir() || metadata.uid() != 0 || metadata.mode() & 0o777 != 0o755 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "unsafe control directory",
        ));
    }
    Ok(())
}

fn remove_stale_socket(path: &Path) -> io::Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !metadata.file_type().is_socket() || UnixStream::connect(path).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "active or unsafe broker socket",
        ));
    }
    fs::remove_file(path)
}

fn chown_socket(path: &Path, uid: u32, gid: u32) -> io::Result<()> {
    let path = CString::new(path.as_os_str().as_encoded_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid socket path"))?;
    if unsafe { libc::chown(path.as_ptr(), uid, gid) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct PeerCredentials {
    uid: u32,
    gid: u32,
    pid: i32,
}

#[cfg(target_os = "linux")]
fn peer_credentials(stream: &UnixStream) -> io::Result<PeerCredentials> {
    let mut credentials = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut length,
        )
    };
    if result != 0 || length as usize != std::mem::size_of::<libc::ucred>() {
        return Err(io::Error::last_os_error());
    }
    Ok(PeerCredentials {
        uid: credentials.uid,
        gid: credentials.gid,
        pid: credentials.pid,
    })
}

#[cfg(not(target_os = "linux"))]
fn peer_credentials(stream: &UnixStream) -> io::Result<PeerCredentials> {
    let mut uid = 0;
    let mut gid = 0;
    if unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(PeerCredentials { uid, gid, pid: 0 })
}
