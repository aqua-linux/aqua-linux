use std::collections::HashSet;
use std::fmt;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::wifi_control::{WifiPassphrase, WifiSsid};

pub const PROTOCOL_VERSION: &str = "AQUA-NETWORK/1";
pub const FIXED_INTERFACE: &str = "eth0";
pub const FIXED_WIFI_INTERFACE: &str = "wlan0";
pub const MAX_REQUEST_BYTES: usize = 256;
pub const MAX_RESPONSE_BYTES: usize = 512;
pub const MAX_STATE_BYTES: usize = 4096;
pub const NETWORK_BROKER_SOCKET_PATH: &str = "/run/aqua-network/control.sock";
const CLIENT_IO_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkBrokerOperation {
    Status,
    RenewDhcp,
}

impl NetworkBrokerOperation {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Status => "STATUS",
            Self::RenewDhcp => "RENEW_DHCP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkBrokerRequest {
    pub operation: NetworkBrokerOperation,
}

impl NetworkBrokerRequest {
    pub fn encode(self) -> String {
        format!(
            "{} {} {}\n",
            PROTOCOL_VERSION,
            self.operation.wire_name(),
            FIXED_INTERFACE
        )
    }
}

pub enum WifiBrokerRequest {
    Status,
    Connect {
        ssid: WifiSsid,
        passphrase: WifiPassphrase,
    },
    Reconnect,
    Disconnect,
}

impl fmt::Debug for WifiBrokerRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Status => "WifiBrokerRequest::Status",
            Self::Connect { .. } => "WifiBrokerRequest::Connect([redacted])",
            Self::Reconnect => "WifiBrokerRequest::Reconnect",
            Self::Disconnect => "WifiBrokerRequest::Disconnect",
        })
    }
}

#[derive(Debug)]
pub enum AuthenticatedNetworkRequest {
    Ethernet(NetworkBrokerRequest),
    Wifi(WifiBrokerRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSupervisorState {
    pub state: String,
    pub interface: String,
    pub attempts: u8,
    pub restarts: u8,
    pub failure: String,
    pub client_pid: Option<u32>,
    pub lease_ready: bool,
    pub route_ready: bool,
    pub dns_ready: bool,
}

impl NetworkSupervisorState {
    pub fn authoritative_ready(&self) -> bool {
        self.state == "running"
            && self.interface == FIXED_INTERFACE
            && self.client_pid.is_some()
            && self.lease_ready
            && self.route_ready
            && self.dns_ready
    }

    pub fn status_response(&self) -> String {
        format!(
            "{} OK operation=status interface={} state={} attempts={} restarts={} failure={} lease={} route={} dns={} management=true\n",
            PROTOCOL_VERSION,
            self.interface,
            self.state,
            self.attempts,
            self.restarts,
            self.failure,
            self.lease_ready,
            self.route_ready,
            self.dns_ready
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkBrokerError {
    RequestTooLarge,
    InvalidUtf8,
    InvalidRequest,
    UnsupportedVersion,
    UnsupportedOperation,
    InvalidInterface,
    InvalidCredential,
    StateTooLarge,
    InvalidState,
}

impl fmt::Display for NetworkBrokerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RequestTooLarge => "request-too-large",
            Self::InvalidUtf8 => "invalid-utf8",
            Self::InvalidRequest => "invalid-request",
            Self::UnsupportedVersion => "unsupported-version",
            Self::UnsupportedOperation => "unsupported-operation",
            Self::InvalidInterface => "invalid-interface",
            Self::InvalidCredential => "invalid-credential",
            Self::StateTooLarge => "state-too-large",
            Self::InvalidState => "invalid-state",
        })
    }
}

pub fn parse_authenticated_request(
    bytes: &[u8],
) -> Result<AuthenticatedNetworkRequest, NetworkBrokerError> {
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(NetworkBrokerError::RequestTooLarge);
    }
    let line = std::str::from_utf8(bytes).map_err(|_| NetworkBrokerError::InvalidUtf8)?;
    let line = line
        .strip_suffix('\n')
        .ok_or(NetworkBrokerError::InvalidRequest)?;
    if line.contains(['\r', '\n', '\0']) {
        return Err(NetworkBrokerError::InvalidRequest);
    }
    let fields = line.split(' ').collect::<Vec<_>>();
    if fields.len() < 3 || fields.iter().any(|field| field.is_empty()) {
        return Err(NetworkBrokerError::InvalidRequest);
    }
    if fields[0] != PROTOCOL_VERSION {
        return Err(NetworkBrokerError::UnsupportedVersion);
    }
    match fields[1] {
        "STATUS" | "RENEW_DHCP" => parse_request(bytes).map(AuthenticatedNetworkRequest::Ethernet),
        "WIFI_STATUS" if fields.len() == 3 => {
            validate_wifi_interface(fields[2])?;
            Ok(AuthenticatedNetworkRequest::Wifi(WifiBrokerRequest::Status))
        }
        "WIFI_DISCONNECT" if fields.len() == 3 => {
            validate_wifi_interface(fields[2])?;
            Ok(AuthenticatedNetworkRequest::Wifi(
                WifiBrokerRequest::Disconnect,
            ))
        }
        "WIFI_RECONNECT" if fields.len() == 3 => {
            validate_wifi_interface(fields[2])?;
            Ok(AuthenticatedNetworkRequest::Wifi(
                WifiBrokerRequest::Reconnect,
            ))
        }
        "WIFI_CONNECT" if fields.len() == 5 => {
            validate_wifi_interface(fields[2])?;
            let ssid = decode_hex(fields[3]).and_then(|value| {
                WifiSsid::new(value).map_err(|_| NetworkBrokerError::InvalidCredential)
            })?;
            let mut passphrase_bytes = decode_hex(fields[4])?;
            let passphrase = WifiPassphrase::from_bytes(&passphrase_bytes)
                .map_err(|_| NetworkBrokerError::InvalidCredential);
            wipe_bytes(&mut passphrase_bytes);
            let passphrase = passphrase?;
            Ok(AuthenticatedNetworkRequest::Wifi(
                WifiBrokerRequest::Connect { ssid, passphrase },
            ))
        }
        "WIFI_STATUS" | "WIFI_DISCONNECT" | "WIFI_RECONNECT" | "WIFI_CONNECT" => {
            Err(NetworkBrokerError::InvalidRequest)
        }
        _ => Err(NetworkBrokerError::UnsupportedOperation),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiBrokerOperation {
    Status,
    Reconnect,
    Disconnect,
}

impl WifiBrokerOperation {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Status => "WIFI_STATUS",
            Self::Reconnect => "WIFI_RECONNECT",
            Self::Disconnect => "WIFI_DISCONNECT",
        }
    }

    const fn response_name(self) -> &'static str {
        match self {
            Self::Status => "wifi-status",
            Self::Reconnect => "wifi-reconnect",
            Self::Disconnect => "wifi-disconnect",
        }
    }

    pub fn encode(self) -> String {
        format!(
            "{PROTOCOL_VERSION} {} {FIXED_WIFI_INTERFACE}\n",
            self.wire_name()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiBrokerStatus {
    pub state: String,
    pub network_id: Option<u16>,
    pub authoritative: bool,
}

impl WifiBrokerStatus {
    pub fn connected(&self) -> bool {
        self.state == "completed" && self.network_id.is_some() && self.authoritative
    }
}

#[derive(Debug)]
pub enum WifiBrokerClientError {
    Io(io::Error),
    ResponseTooLarge,
    InvalidResponse,
    Rejected(String),
}

impl fmt::Display for WifiBrokerClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "broker-io-{}", error.kind()),
            Self::ResponseTooLarge => formatter.write_str("broker-response-too-large"),
            Self::InvalidResponse => formatter.write_str("invalid-broker-response"),
            Self::Rejected(reason) => write!(formatter, "broker-rejected-{reason}"),
        }
    }
}

impl std::error::Error for WifiBrokerClientError {}

impl From<io::Error> for WifiBrokerClientError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn request_wifi_broker(
    socket_path: &Path,
    operation: WifiBrokerOperation,
) -> Result<WifiBrokerStatus, WifiBrokerClientError> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.set_read_timeout(Some(CLIENT_IO_TIMEOUT))?;
    stream.set_write_timeout(Some(CLIENT_IO_TIMEOUT))?;
    stream.write_all(operation.encode().as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut response = Vec::with_capacity(MAX_RESPONSE_BYTES + 1);
    Read::by_ref(&mut stream)
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut response)?;
    parse_wifi_broker_response(&response, operation)
}

pub fn parse_wifi_broker_response(
    bytes: &[u8],
    operation: WifiBrokerOperation,
) -> Result<WifiBrokerStatus, WifiBrokerClientError> {
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(WifiBrokerClientError::ResponseTooLarge);
    }
    let line = std::str::from_utf8(bytes).map_err(|_| WifiBrokerClientError::InvalidResponse)?;
    let line = line
        .strip_suffix('\n')
        .ok_or(WifiBrokerClientError::InvalidResponse)?;
    if line.contains(['\r', '\n', '\0']) {
        return Err(WifiBrokerClientError::InvalidResponse);
    }
    if let Some(reason) = line.strip_prefix(&format!("{PROTOCOL_VERSION} ERROR ")) {
        if valid_response_value(reason) {
            return Err(WifiBrokerClientError::Rejected(reason.to_owned()));
        }
        return Err(WifiBrokerClientError::InvalidResponse);
    }
    let fields = line.split(' ').collect::<Vec<_>>();
    if fields.len() < 5 || fields[0] != PROTOCOL_VERSION || fields[1] != "OK" {
        return Err(WifiBrokerClientError::InvalidResponse);
    }
    let mut values = std::collections::HashMap::new();
    for field in &fields[2..] {
        let (key, value) = field
            .split_once('=')
            .ok_or(WifiBrokerClientError::InvalidResponse)?;
        if !valid_response_key(key)
            || !valid_response_value(value)
            || values.insert(key, value).is_some()
        {
            return Err(WifiBrokerClientError::InvalidResponse);
        }
    }
    if values.get("operation") != Some(&operation.response_name())
        || values.get("interface") != Some(&FIXED_WIFI_INTERFACE)
    {
        return Err(WifiBrokerClientError::InvalidResponse);
    }
    let allowed = match operation {
        WifiBrokerOperation::Status => [
            "operation",
            "interface",
            "state",
            "network_id",
            "authoritative",
        ]
        .as_slice(),
        WifiBrokerOperation::Reconnect => [
            "operation",
            "interface",
            "network_id",
            "authoritative",
            "credential_saved",
        ]
        .as_slice(),
        WifiBrokerOperation::Disconnect => ["operation", "interface", "authoritative"].as_slice(),
    };
    if values.keys().any(|key| !allowed.contains(key)) {
        return Err(WifiBrokerClientError::InvalidResponse);
    }
    let authoritative = match values.get("authoritative") {
        Some(&"true") => true,
        Some(&"false") => false,
        _ => return Err(WifiBrokerClientError::InvalidResponse),
    };
    let network_id = match values.get("network_id") {
        Some(&"none") | None => None,
        Some(value) => Some(
            value
                .parse::<u16>()
                .map_err(|_| WifiBrokerClientError::InvalidResponse)
                .and_then(|value| {
                    (value <= crate::wifi_control::MAX_WIFI_NETWORK_ID)
                        .then_some(value)
                        .ok_or(WifiBrokerClientError::InvalidResponse)
                })?,
        ),
    };
    match operation {
        WifiBrokerOperation::Status
            if !values.contains_key("state") || !values.contains_key("network_id") =>
        {
            return Err(WifiBrokerClientError::InvalidResponse);
        }
        WifiBrokerOperation::Reconnect
            if network_id.is_none()
                || !authoritative
                || values.get("credential_saved") != Some(&"true") =>
        {
            return Err(WifiBrokerClientError::InvalidResponse);
        }
        WifiBrokerOperation::Disconnect if !authoritative => {
            return Err(WifiBrokerClientError::InvalidResponse);
        }
        _ => {}
    }
    let state = match values.get("state").copied() {
        Some(state) => state,
        None => match operation {
            WifiBrokerOperation::Reconnect => "completed",
            WifiBrokerOperation::Disconnect => "disconnected",
            WifiBrokerOperation::Status => return Err(WifiBrokerClientError::InvalidResponse),
        },
    };
    if !matches!(
        state,
        "disconnected"
            | "scanning"
            | "authenticating"
            | "associating"
            | "associated"
            | "four-way-handshake"
            | "group-handshake"
            | "completed"
            | "inactive"
            | "interface-disabled"
    ) {
        return Err(WifiBrokerClientError::InvalidResponse);
    }
    Ok(WifiBrokerStatus {
        state: state.to_owned(),
        network_id,
        authoritative,
    })
}

fn valid_response_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_response_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

impl std::error::Error for NetworkBrokerError {}

pub fn parse_request(bytes: &[u8]) -> Result<NetworkBrokerRequest, NetworkBrokerError> {
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(NetworkBrokerError::RequestTooLarge);
    }
    let line = std::str::from_utf8(bytes).map_err(|_| NetworkBrokerError::InvalidUtf8)?;
    let line = line
        .strip_suffix('\n')
        .ok_or(NetworkBrokerError::InvalidRequest)?;
    if line.contains(['\r', '\n', '\0']) {
        return Err(NetworkBrokerError::InvalidRequest);
    }
    let fields = line.split(' ').collect::<Vec<_>>();
    if fields.len() != 3 || fields.iter().any(|field| field.is_empty()) {
        return Err(NetworkBrokerError::InvalidRequest);
    }
    if fields[0] != PROTOCOL_VERSION {
        return Err(NetworkBrokerError::UnsupportedVersion);
    }
    if fields[2] != FIXED_INTERFACE {
        return Err(NetworkBrokerError::InvalidInterface);
    }
    let operation = match fields[1] {
        "STATUS" => NetworkBrokerOperation::Status,
        "RENEW_DHCP" => NetworkBrokerOperation::RenewDhcp,
        _ => return Err(NetworkBrokerError::UnsupportedOperation),
    };
    Ok(NetworkBrokerRequest { operation })
}

fn validate_wifi_interface(interface: &str) -> Result<(), NetworkBrokerError> {
    if interface == FIXED_WIFI_INTERFACE {
        Ok(())
    } else {
        Err(NetworkBrokerError::InvalidInterface)
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, NetworkBrokerError> {
    if value.is_empty() || value.len() % 2 != 0 {
        return Err(NetworkBrokerError::InvalidCredential);
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = decode_hex_digit(pair[0])?;
            let low = decode_hex_digit(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn decode_hex_digit(value: u8) -> Result<u8, NetworkBrokerError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(NetworkBrokerError::InvalidCredential),
    }
}

fn wipe_bytes(bytes: &mut [u8]) {
    for byte in bytes {
        unsafe { std::ptr::write_volatile(byte, 0) };
    }
}

pub fn parse_supervisor_state(bytes: &[u8]) -> Result<NetworkSupervisorState, NetworkBrokerError> {
    if bytes.len() > MAX_STATE_BYTES {
        return Err(NetworkBrokerError::StateTooLarge);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| NetworkBrokerError::InvalidState)?;
    let mut seen = HashSet::new();
    let mut state = None;
    let mut interface = None;
    let mut attempts = None;
    let mut restarts = None;
    let mut failure = None;
    let mut client_pid = None;
    let mut lease_ready = None;
    let mut route_ready = None;
    let mut dns_ready = None;

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            return Err(NetworkBrokerError::InvalidState);
        };
        if !seen.insert(key) {
            return Err(NetworkBrokerError::InvalidState);
        }
        match key {
            "state" => state = Some(parse_state_name(value)?),
            "interface" if value == FIXED_INTERFACE => interface = Some(value.to_owned()),
            "interface" => return Err(NetworkBrokerError::InvalidState),
            "attempts" => attempts = Some(parse_count(value)?),
            "restarts" => restarts = Some(parse_count(value)?),
            "failure" => failure = Some(parse_failure(value)?),
            "client_pid" => {
                client_pid = Some(if value.is_empty() {
                    None
                } else {
                    Some(parse_pid(value)?)
                });
            }
            "lease_ready" => lease_ready = Some(parse_bool(value)?),
            "route_ready" => route_ready = Some(parse_bool(value)?),
            "dns_ready" => dns_ready = Some(parse_bool(value)?),
            "product"
            | "max_restarts"
            | "service_owner_uid"
            | "policy_owner"
            | "legacy_owner_disabled"
            | "settings_management"
            | "wifi_packaged" => {}
            _ => return Err(NetworkBrokerError::InvalidState),
        }
    }

    Ok(NetworkSupervisorState {
        state: state.ok_or(NetworkBrokerError::InvalidState)?,
        interface: interface.ok_or(NetworkBrokerError::InvalidState)?,
        attempts: attempts.ok_or(NetworkBrokerError::InvalidState)?,
        restarts: restarts.ok_or(NetworkBrokerError::InvalidState)?,
        failure: failure.ok_or(NetworkBrokerError::InvalidState)?,
        client_pid: client_pid.ok_or(NetworkBrokerError::InvalidState)?,
        lease_ready: lease_ready.ok_or(NetworkBrokerError::InvalidState)?,
        route_ready: route_ready.ok_or(NetworkBrokerError::InvalidState)?,
        dns_ready: dns_ready.ok_or(NetworkBrokerError::InvalidState)?,
    })
}

fn parse_count(value: &str) -> Result<u8, NetworkBrokerError> {
    value
        .parse::<u8>()
        .map_err(|_| NetworkBrokerError::InvalidState)
}

fn parse_pid(value: &str) -> Result<u32, NetworkBrokerError> {
    let pid = value
        .parse::<u32>()
        .map_err(|_| NetworkBrokerError::InvalidState)?;
    if pid <= 1 {
        return Err(NetworkBrokerError::InvalidState);
    }
    Ok(pid)
}

fn parse_bool(value: &str) -> Result<bool, NetworkBrokerError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(NetworkBrokerError::InvalidState),
    }
}

fn parse_state_name(value: &str) -> Result<String, NetworkBrokerError> {
    match value {
        "starting" | "running" | "restarting" | "stopped" | "degraded" | "disabled" | "blocked"
        | "policy-ready" => Ok(value.to_owned()),
        _ => Err(NetworkBrokerError::InvalidState),
    }
}

fn parse_failure(value: &str) -> Result<String, NetworkBrokerError> {
    if value.len() <= 32
        && !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
    {
        Ok(value.to_owned())
    } else {
        Err(NetworkBrokerError::InvalidState)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const READY_STATE: &str = "product=Aqua Linux\nstate=running\ninterface=eth0\nattempts=2\nrestarts=1\nmax_restarts=3\nfailure=none\nclient_pid=42\nlease_ready=true\nroute_ready=true\ndns_ready=true\nservice_owner_uid=0\npolicy_owner=aqua-network-service-supervisor\nlegacy_owner_disabled=true\nsettings_management=false\nwifi_packaged=false\n";

    #[test]
    fn protocol_accepts_only_versioned_typed_fixed_interface_requests() {
        assert_eq!(
            parse_request(b"AQUA-NETWORK/1 STATUS eth0\n"),
            Ok(NetworkBrokerRequest {
                operation: NetworkBrokerOperation::Status,
            })
        );
        assert_eq!(
            parse_request(b"AQUA-NETWORK/1 RENEW_DHCP eth0\n"),
            Ok(NetworkBrokerRequest {
                operation: NetworkBrokerOperation::RenewDhcp,
            })
        );
        assert_eq!(
            parse_request(b"AQUA-NETWORK/1 STATUS wlan0\n"),
            Err(NetworkBrokerError::InvalidInterface)
        );
        assert_eq!(
            parse_request(b"AQUA-NETWORK/1 EXEC eth0\n"),
            Err(NetworkBrokerError::UnsupportedOperation)
        );
        assert_eq!(
            parse_request(b"AQUA-NETWORK/1 STATUS eth0 extra\n"),
            Err(NetworkBrokerError::InvalidRequest)
        );
    }

    #[test]
    fn protocol_rejects_unbounded_or_multiline_input() {
        assert_eq!(
            parse_request(&[b'a'; MAX_REQUEST_BYTES + 1]),
            Err(NetworkBrokerError::RequestTooLarge)
        );
        assert_eq!(
            parse_request(b"AQUA-NETWORK/1 STATUS eth0\nEXEC root\n"),
            Err(NetworkBrokerError::InvalidRequest)
        );
        assert_eq!(
            parse_request(b"AQUA-NETWORK/1 STATUS eth0"),
            Err(NetworkBrokerError::InvalidRequest)
        );
    }

    #[test]
    fn authenticated_protocol_types_and_redacts_wifi_credentials() {
        let request = parse_authenticated_request(
            b"AQUA-NETWORK/1 WIFI_CONNECT wlan0 41717561204c6162 636f727265637420686f727365\n",
        )
        .expect("valid typed Wi-Fi request");
        let AuthenticatedNetworkRequest::Wifi(WifiBrokerRequest::Connect { ssid, passphrase }) =
            request
        else {
            panic!("expected Wi-Fi connect request");
        };
        assert_eq!(ssid.bytes(), b"Aqua Lab");
        assert_eq!(passphrase.with_bytes(|value| value.len()), 13);
        let debug = format!("{passphrase:?}");
        assert!(!debug.contains("correct horse"));

        assert!(matches!(
            parse_authenticated_request(b"AQUA-NETWORK/1 WIFI_STATUS wlan0\n"),
            Ok(AuthenticatedNetworkRequest::Wifi(WifiBrokerRequest::Status))
        ));
        assert!(matches!(
            parse_authenticated_request(b"AQUA-NETWORK/1 WIFI_DISCONNECT wlan0\n"),
            Ok(AuthenticatedNetworkRequest::Wifi(
                WifiBrokerRequest::Disconnect
            ))
        ));
        assert!(matches!(
            parse_authenticated_request(b"AQUA-NETWORK/1 WIFI_RECONNECT wlan0\n"),
            Ok(AuthenticatedNetworkRequest::Wifi(
                WifiBrokerRequest::Reconnect
            ))
        ));
    }

    #[test]
    fn settings_wifi_client_accepts_only_typed_broker_responses() {
        let status = parse_wifi_broker_response(
            b"AQUA-NETWORK/1 OK operation=wifi-status interface=wlan0 state=completed network_id=7 authoritative=true\n",
            WifiBrokerOperation::Status,
        )
        .expect("valid status");
        assert!(status.connected());
        assert_eq!(status.network_id, Some(7));

        let disconnected = parse_wifi_broker_response(
            b"AQUA-NETWORK/1 OK operation=wifi-disconnect interface=wlan0 authoritative=true\n",
            WifiBrokerOperation::Disconnect,
        )
        .expect("valid disconnect");
        assert!(!disconnected.connected());
        assert_eq!(disconnected.state, "disconnected");

        let reconnected = parse_wifi_broker_response(
            b"AQUA-NETWORK/1 OK operation=wifi-reconnect interface=wlan0 network_id=8 authoritative=true credential_saved=true\n",
            WifiBrokerOperation::Reconnect,
        )
        .expect("valid reconnect");
        assert!(reconnected.connected());

        assert!(matches!(
            parse_wifi_broker_response(
                b"AQUA-NETWORK/1 ERROR credential-read-failed\n",
                WifiBrokerOperation::Reconnect,
            ),
            Err(WifiBrokerClientError::Rejected(reason)) if reason == "credential-read-failed"
        ));
        assert!(matches!(
            parse_wifi_broker_response(
                b"AQUA-NETWORK/1 OK operation=wifi-status interface=wlan0 state=completed network_id=7 authoritative=true injected=value\n",
                WifiBrokerOperation::Status,
            ),
            Err(WifiBrokerClientError::InvalidResponse)
        ));
    }

    #[test]
    fn authenticated_protocol_rejects_wifi_injection_and_invalid_secrets() {
        assert!(matches!(
            parse_authenticated_request(
                b"AQUA-NETWORK/1 WIFI_CONNECT wlan1 41717561 636f727265637431\n"
            ),
            Err(NetworkBrokerError::InvalidInterface)
        ));
        assert!(matches!(
            parse_authenticated_request(b"AQUA-NETWORK/1 WIFI_CONNECT wlan0 41717561 73686f7274\n"),
            Err(NetworkBrokerError::InvalidCredential)
        ));
        assert!(matches!(
            parse_authenticated_request(
                b"AQUA-NETWORK/1 WIFI_CONNECT wlan0 41717561 636f727265637431 extra\n"
            ),
            Err(NetworkBrokerError::InvalidRequest)
        ));
        assert!(matches!(
            parse_authenticated_request(b"AQUA-NETWORK/1 WIFI_STATUS wlan0\nEXEC root\n"),
            Err(NetworkBrokerError::InvalidRequest)
        ));
    }

    #[test]
    fn supervisor_state_is_bounded_typed_and_authoritative() {
        let state = parse_supervisor_state(READY_STATE.as_bytes()).expect("valid state");
        assert!(state.authoritative_ready());
        assert_eq!(state.client_pid, Some(42));
        assert!(state.status_response().contains("management=true"));

        let duplicate = READY_STATE.replace("state=running", "state=running\nstate=degraded");
        assert_eq!(
            parse_supervisor_state(duplicate.as_bytes()),
            Err(NetworkBrokerError::InvalidState)
        );
        assert_eq!(
            parse_supervisor_state(&[b'x'; MAX_STATE_BYTES + 1]),
            Err(NetworkBrokerError::StateTooLarge)
        );
    }

    #[test]
    fn unavailable_state_cannot_authorize_renewal() {
        let input = READY_STATE
            .replace("state=running", "state=degraded")
            .replace("client_pid=42", "client_pid=")
            .replace("route_ready=true", "route_ready=false");
        let state = parse_supervisor_state(input.as_bytes()).expect("valid degraded state");
        assert!(!state.authoritative_ready());
    }
}
