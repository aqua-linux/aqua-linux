use aqua_service_adapters::network_broker::{
    parse_request, parse_supervisor_state, NetworkBrokerOperation, NetworkBrokerRequest,
    NetworkSupervisorState, FIXED_INTERFACE, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES,
    MAX_STATE_BYTES, PROTOCOL_VERSION,
};
use std::env;
use std::ffi::CString;
use std::fs;
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, SystemTime};

const SOCKET_PATH: &str = "/run/aqua-network/control.sock";
const STATE_PATH: &str = "/run/aqua-network/network-service-supervisor.state";
const READY_PATH: &str = "/run/aqua-network/lease.ready";
const AQUA_UID: u32 = 1000;
const AQUA_GID: u32 = 1000;
const IO_TIMEOUT: Duration = Duration::from_secs(2);
const RENEW_TIMEOUT: Duration = Duration::from_secs(8);

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
        "[AQUA-NETWORK] stage=network-privilege-broker status=ready socket={} owner_uid=0 client_uid={} interface={} operations=status,renew-dhcp arbitrary_commands=false arbitrary_paths=false",
        SOCKET_PATH, AQUA_UID, FIXED_INTERFACE
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
    let request = match parse_request(&bytes) {
        Ok(request) => request,
        Err(error) => {
            write_response(&mut stream, &format!("{PROTOCOL_VERSION} ERROR {error}\n"))?;
            return Ok(());
        }
    };
    let state = read_state(Path::new(STATE_PATH))?;
    match request.operation {
        NetworkBrokerOperation::Status => write_response(&mut stream, &state.status_response())?,
        NetworkBrokerOperation::RenewDhcp => renew_dhcp(&mut stream, &state)?,
    }
    Ok(())
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
