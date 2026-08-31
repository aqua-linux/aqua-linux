use std::collections::HashSet;
use std::fmt;

pub const WIFI_CONTROL_PROTOCOL: &str = "AQUA-WIFI-CONTROL/1";
pub const WIFI_CREDENTIAL_RECORD_VERSION: &str = "AQUA-WIFI-CREDENTIAL/1";
pub const WIFI_CREDENTIAL_DIRECTORY: &str = "/var/lib/aqua-network";
pub const WIFI_CREDENTIAL_PATH: &str = "/var/lib/aqua-network/wifi.psk";
pub const WIFI_CREDENTIAL_TEMP_PATH: &str = "/var/lib/aqua-network/.wifi.psk.new";
pub const WIFI_CREDENTIAL_DIRECTORY_MODE: u32 = 0o700;
pub const WIFI_CREDENTIAL_FILE_MODE: u32 = 0o600;
pub const MAX_WIFI_SSID_BYTES: usize = 32;
pub const MIN_WIFI_PASSPHRASE_BYTES: usize = 8;
pub const MAX_WIFI_PASSPHRASE_BYTES: usize = 63;
pub const MAX_WIFI_NETWORK_ID: u16 = 4095;
pub const MAX_WIFI_CONTROL_COMMAND_BYTES: usize = 192;
pub const MAX_WIFI_CONTROL_RESPONSE_BYTES: usize = 4096;
pub const MAX_WIFI_CREDENTIAL_RECORD_BYTES: usize = 256;

#[derive(Clone, PartialEq, Eq)]
pub struct WifiSsid(Vec<u8>);

impl WifiSsid {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Result<Self, WifiControlError> {
        let bytes = bytes.into();
        if bytes.is_empty() || bytes.len() > MAX_WIFI_SSID_BYTES {
            return Err(WifiControlError::InvalidSsid);
        }
        Ok(Self(bytes))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for WifiSsid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WifiSsid")
            .field("bytes", &"[redacted]")
            .field("length", &self.0.len())
            .finish()
    }
}

pub struct WifiPassphrase {
    bytes: [u8; MAX_WIFI_PASSPHRASE_BYTES],
    length: usize,
}

impl WifiPassphrase {
    pub fn new(value: &str) -> Result<Self, WifiControlError> {
        Self::from_bytes(value.as_bytes())
    }

    pub fn from_bytes(value: &[u8]) -> Result<Self, WifiControlError> {
        if !(MIN_WIFI_PASSPHRASE_BYTES..=MAX_WIFI_PASSPHRASE_BYTES).contains(&value.len())
            || !value.iter().all(|byte| (0x20..=0x7e).contains(byte))
        {
            return Err(WifiControlError::InvalidPassphrase);
        }
        let mut bytes = [0; MAX_WIFI_PASSPHRASE_BYTES];
        bytes[..value.len()].copy_from_slice(value);
        Ok(Self {
            bytes,
            length: value.len(),
        })
    }

    pub fn with_bytes<T>(&self, operation: impl FnOnce(&[u8]) -> T) -> T {
        operation(&self.bytes[..self.length])
    }
}

impl fmt::Debug for WifiPassphrase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WifiPassphrase")
            .field("bytes", &"[redacted]")
            .field("length", &self.length)
            .finish()
    }
}

impl Drop for WifiPassphrase {
    fn drop(&mut self) {
        for byte in &mut self.bytes {
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
        self.length = 0;
    }
}

pub struct WifiPsk([u8; 32]);

impl WifiPsk {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn securely_matches(&self, expected: &[u8; 32]) -> bool {
        self.0
            .iter()
            .zip(expected)
            .fold(0_u8, |difference, (actual, expected)| {
                difference | (actual ^ expected)
            })
            == 0
    }

    pub fn from_hex(value: &str) -> Result<Self, WifiControlError> {
        if value.len() != 64 {
            return Err(WifiControlError::InvalidPsk);
        }
        let mut bytes = [0; 32];
        for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
            let high = decode_hex(pair[0]).map_err(|_| WifiControlError::InvalidPsk)?;
            let low = decode_hex(pair[1]).map_err(|_| WifiControlError::InvalidPsk)?;
            bytes[index] = (high << 4) | low;
        }
        Ok(Self(bytes))
    }

    fn write_hex(&self, output: &mut String) {
        for byte in self.0 {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
}

impl fmt::Debug for WifiPsk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WifiPsk([redacted])")
    }
}

impl Drop for WifiPsk {
    fn drop(&mut self) {
        for byte in &mut self.0 {
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    Wpa2Personal,
}

impl WifiSecurity {
    const fn wire_name(self) -> &'static str {
        match self {
            Self::Wpa2Personal => "wpa2-personal",
        }
    }
}

pub struct WifiCredentialRecord {
    ssid: WifiSsid,
    security: WifiSecurity,
    psk: WifiPsk,
}

impl WifiCredentialRecord {
    pub fn new(ssid: WifiSsid, security: WifiSecurity, psk: WifiPsk) -> Self {
        Self {
            ssid,
            security,
            psk,
        }
    }

    pub fn ssid(&self) -> &WifiSsid {
        &self.ssid
    }

    pub const fn security(&self) -> WifiSecurity {
        self.security
    }

    pub fn psk(&self) -> &WifiPsk {
        &self.psk
    }

    pub fn encode(&self) -> WifiCredentialPayload {
        let mut record = Vec::with_capacity(MAX_WIFI_CREDENTIAL_RECORD_BYTES);
        record.extend_from_slice(WIFI_CREDENTIAL_RECORD_VERSION.as_bytes());
        record.extend_from_slice(b"\nsecurity=");
        record.extend_from_slice(self.security.wire_name().as_bytes());
        record.extend_from_slice(b"\nssid=");
        write_hex_bytes(self.ssid.bytes(), &mut record);
        record.extend_from_slice(b"\npsk=");
        write_hex_bytes(&self.psk.0, &mut record);
        record.push(b'\n');
        debug_assert!(record.len() <= MAX_WIFI_CREDENTIAL_RECORD_BYTES);
        WifiCredentialPayload(record)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, WifiControlError> {
        if bytes.len() > MAX_WIFI_CREDENTIAL_RECORD_BYTES {
            return Err(WifiControlError::CredentialRecordTooLarge);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| WifiControlError::InvalidCredential)?;
        if text.contains(['\r', '\0']) || !text.ends_with('\n') {
            return Err(WifiControlError::InvalidCredential);
        }
        let lines = text.lines().collect::<Vec<_>>();
        if lines.len() != 4 || lines[0] != WIFI_CREDENTIAL_RECORD_VERSION {
            return Err(WifiControlError::InvalidCredential);
        }
        let security = match lines[1] {
            "security=wpa2-personal" => WifiSecurity::Wpa2Personal,
            _ => return Err(WifiControlError::InvalidCredential),
        };
        let ssid_hex = lines[2]
            .strip_prefix("ssid=")
            .ok_or(WifiControlError::InvalidCredential)?;
        let psk_hex = lines[3]
            .strip_prefix("psk=")
            .ok_or(WifiControlError::InvalidCredential)?;
        if ssid_hex.len() % 2 != 0 {
            return Err(WifiControlError::InvalidCredential);
        }
        let ssid = ssid_hex
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| Ok((decode_hex(pair[0])? << 4) | decode_hex(pair[1])?))
            .collect::<Result<Vec<_>, WifiControlError>>()?;
        Ok(Self::new(
            WifiSsid::new(ssid)?,
            security,
            WifiPsk::from_hex(psk_hex)?,
        ))
    }
}

impl fmt::Debug for WifiCredentialRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WifiCredentialRecord")
            .field("ssid", &self.ssid)
            .field("security", &self.security)
            .field("psk", &self.psk)
            .finish()
    }
}

pub struct WifiCredentialPayload(Vec<u8>);

impl WifiCredentialPayload {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for WifiCredentialPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WifiCredentialPayload")
            .field("bytes", &"[redacted]")
            .field("length", &self.0.len())
            .finish()
    }
}

impl Drop for WifiCredentialPayload {
    fn drop(&mut self) {
        for byte in &mut self.0 {
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WifiCredentialMetadata {
    pub directory_uid: u32,
    pub directory_mode: u32,
    pub directory_is_symlink: bool,
    pub file_uid: u32,
    pub file_mode: u32,
    pub file_is_regular: bool,
    pub file_is_symlink: bool,
    pub file_bytes: usize,
}

pub fn validate_credential_metadata(
    metadata: WifiCredentialMetadata,
) -> Result<(), WifiControlError> {
    if metadata.directory_uid != 0
        || metadata.directory_mode != WIFI_CREDENTIAL_DIRECTORY_MODE
        || metadata.directory_is_symlink
        || metadata.file_uid != 0
        || metadata.file_mode != WIFI_CREDENTIAL_FILE_MODE
        || !metadata.file_is_regular
        || metadata.file_is_symlink
    {
        return Err(WifiControlError::UnsafeCredentialStorage);
    }
    if metadata.file_bytes == 0 || metadata.file_bytes > MAX_WIFI_CREDENTIAL_RECORD_BYTES {
        return Err(WifiControlError::CredentialRecordTooLarge);
    }
    Ok(())
}

pub enum WifiControlRequest<'a> {
    Status,
    Scan,
    AddNetwork,
    SetSsid { network_id: u16, ssid: &'a WifiSsid },
    SetPsk { network_id: u16, psk: &'a WifiPsk },
    SetWpa2Personal { network_id: u16 },
    EnableNetwork { network_id: u16 },
    SelectNetwork { network_id: u16 },
    RemoveNetwork { network_id: u16 },
    Disconnect,
}

impl fmt::Debug for WifiControlRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Status => "WifiControlRequest::Status",
            Self::Scan => "WifiControlRequest::Scan",
            Self::AddNetwork => "WifiControlRequest::AddNetwork",
            Self::SetSsid { .. } => "WifiControlRequest::SetSsid([redacted])",
            Self::SetPsk { .. } => "WifiControlRequest::SetPsk([redacted])",
            Self::SetWpa2Personal { .. } => "WifiControlRequest::SetWpa2Personal",
            Self::EnableNetwork { .. } => "WifiControlRequest::EnableNetwork",
            Self::SelectNetwork { .. } => "WifiControlRequest::SelectNetwork",
            Self::RemoveNetwork { .. } => "WifiControlRequest::RemoveNetwork",
            Self::Disconnect => "WifiControlRequest::Disconnect",
        })
    }
}

impl WifiControlRequest<'_> {
    pub fn encode(&self) -> Result<WifiControlCommand, WifiControlError> {
        let command = match self {
            Self::Status => "STATUS".to_owned(),
            Self::Scan => "SCAN".to_owned(),
            Self::AddNetwork => "ADD_NETWORK".to_owned(),
            Self::SetSsid { network_id, ssid } => {
                validate_network_id(*network_id)?;
                let mut command = format!("SET_NETWORK {network_id} ssid ");
                write_hex(ssid.bytes(), &mut command);
                command
            }
            Self::SetPsk { network_id, psk } => {
                validate_network_id(*network_id)?;
                let mut command = format!("SET_NETWORK {network_id} psk ");
                psk.write_hex(&mut command);
                command
            }
            Self::SetWpa2Personal { network_id } => {
                validate_network_id(*network_id)?;
                format!("SET_NETWORK {network_id} key_mgmt WPA-PSK")
            }
            Self::EnableNetwork { network_id } => {
                validate_network_id(*network_id)?;
                format!("ENABLE_NETWORK {network_id}")
            }
            Self::SelectNetwork { network_id } => {
                validate_network_id(*network_id)?;
                format!("SELECT_NETWORK {network_id}")
            }
            Self::RemoveNetwork { network_id } => {
                validate_network_id(*network_id)?;
                format!("REMOVE_NETWORK {network_id}")
            }
            Self::Disconnect => "DISCONNECT".to_owned(),
        };
        if command.len() > MAX_WIFI_CONTROL_COMMAND_BYTES {
            return Err(WifiControlError::CommandTooLarge);
        }
        Ok(WifiControlCommand(command.into_bytes()))
    }

    pub fn parse_response(&self, bytes: &[u8]) -> Result<WifiControlResponse, WifiControlError> {
        if bytes.len() > MAX_WIFI_CONTROL_RESPONSE_BYTES {
            return Err(WifiControlError::ResponseTooLarge);
        }
        match self {
            Self::Status => parse_status(bytes).map(WifiControlResponse::Status),
            Self::AddNetwork => parse_network_id(bytes).map(WifiControlResponse::NetworkAdded),
            _ if bytes == b"OK\n" => Ok(WifiControlResponse::Acknowledged),
            _ if bytes == b"FAIL\n" => Err(WifiControlError::SupplicantRejected),
            _ => Err(WifiControlError::InvalidResponse),
        }
    }
}

pub struct WifiControlCommand(Vec<u8>);

impl WifiControlCommand {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for WifiControlCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WifiControlCommand")
            .field("bytes", &"[redacted]")
            .field("length", &self.0.len())
            .finish()
    }
}

impl Drop for WifiControlCommand {
    fn drop(&mut self) {
        for byte in &mut self.0 {
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiAssociationState {
    Disconnected,
    Scanning,
    Authenticating,
    Associating,
    Associated,
    FourWayHandshake,
    GroupHandshake,
    Completed,
    Inactive,
    InterfaceDisabled,
}

impl WifiAssociationState {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Scanning => "scanning",
            Self::Authenticating => "authenticating",
            Self::Associating => "associating",
            Self::Associated => "associated",
            Self::FourWayHandshake => "four-way-handshake",
            Self::GroupHandshake => "group-handshake",
            Self::Completed => "completed",
            Self::Inactive => "inactive",
            Self::InterfaceDisabled => "interface-disabled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WifiControlStatus {
    pub state: WifiAssociationState,
    pub network_id: Option<u16>,
}

impl WifiControlStatus {
    pub const fn authoritative_association(self) -> bool {
        matches!(self.state, WifiAssociationState::Completed) && self.network_id.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiControlResponse {
    Acknowledged,
    NetworkAdded(u16),
    Status(WifiControlStatus),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiControlError {
    InvalidSsid,
    InvalidPassphrase,
    InvalidPsk,
    InvalidNetworkId,
    CommandTooLarge,
    ResponseTooLarge,
    InvalidResponse,
    SupplicantRejected,
    CredentialRecordTooLarge,
    InvalidCredential,
    UnsafeCredentialStorage,
}

impl fmt::Display for WifiControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSsid => "invalid-ssid",
            Self::InvalidPassphrase => "invalid-passphrase",
            Self::InvalidPsk => "invalid-psk",
            Self::InvalidNetworkId => "invalid-network-id",
            Self::CommandTooLarge => "command-too-large",
            Self::ResponseTooLarge => "response-too-large",
            Self::InvalidResponse => "invalid-response",
            Self::SupplicantRejected => "supplicant-rejected",
            Self::CredentialRecordTooLarge => "credential-record-too-large",
            Self::InvalidCredential => "invalid-credential",
            Self::UnsafeCredentialStorage => "unsafe-credential-storage",
        })
    }
}

impl std::error::Error for WifiControlError {}

fn validate_network_id(network_id: u16) -> Result<(), WifiControlError> {
    if network_id > MAX_WIFI_NETWORK_ID {
        return Err(WifiControlError::InvalidNetworkId);
    }
    Ok(())
}

fn parse_network_id(bytes: &[u8]) -> Result<u16, WifiControlError> {
    let text = std::str::from_utf8(bytes).map_err(|_| WifiControlError::InvalidResponse)?;
    let value = text
        .strip_suffix('\n')
        .ok_or(WifiControlError::InvalidResponse)?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(WifiControlError::InvalidResponse);
    }
    let network_id = value
        .parse::<u16>()
        .map_err(|_| WifiControlError::InvalidResponse)?;
    validate_network_id(network_id)?;
    Ok(network_id)
}

fn parse_status(bytes: &[u8]) -> Result<WifiControlStatus, WifiControlError> {
    let text = std::str::from_utf8(bytes).map_err(|_| WifiControlError::InvalidResponse)?;
    if text.contains(['\r', '\0']) || !text.ends_with('\n') {
        return Err(WifiControlError::InvalidResponse);
    }
    let mut seen = HashSet::new();
    let mut state = None;
    let mut network_id = None;
    for (line_count, line) in text.lines().enumerate() {
        if line_count >= 32 || line.len() > 160 {
            return Err(WifiControlError::InvalidResponse);
        }
        let (key, value) = line
            .split_once('=')
            .ok_or(WifiControlError::InvalidResponse)?;
        if key.is_empty()
            || key.len() > 32
            || value.len() > 128
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
            || !seen.insert(key)
        {
            return Err(WifiControlError::InvalidResponse);
        }
        match key {
            "wpa_state" => state = Some(parse_association_state(value)?),
            "id" => network_id = Some(parse_network_id(format!("{value}\n").as_bytes())?),
            _ => {}
        }
    }
    let status = WifiControlStatus {
        state: state.ok_or(WifiControlError::InvalidResponse)?,
        network_id,
    };
    if status.state == WifiAssociationState::Completed && status.network_id.is_none() {
        return Err(WifiControlError::InvalidResponse);
    }
    Ok(status)
}

fn parse_association_state(value: &str) -> Result<WifiAssociationState, WifiControlError> {
    match value {
        "DISCONNECTED" => Ok(WifiAssociationState::Disconnected),
        "SCANNING" => Ok(WifiAssociationState::Scanning),
        "AUTHENTICATING" => Ok(WifiAssociationState::Authenticating),
        "ASSOCIATING" => Ok(WifiAssociationState::Associating),
        "ASSOCIATED" => Ok(WifiAssociationState::Associated),
        "4WAY_HANDSHAKE" => Ok(WifiAssociationState::FourWayHandshake),
        "GROUP_HANDSHAKE" => Ok(WifiAssociationState::GroupHandshake),
        "COMPLETED" => Ok(WifiAssociationState::Completed),
        "INACTIVE" => Ok(WifiAssociationState::Inactive),
        "INTERFACE_DISABLED" => Ok(WifiAssociationState::InterfaceDisabled),
        _ => Err(WifiControlError::InvalidResponse),
    }
}

const HEX: &[u8; 16] = b"0123456789abcdef";

fn write_hex(bytes: &[u8], output: &mut String) {
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
}

fn write_hex_bytes(bytes: &[u8], output: &mut Vec<u8>) {
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize]);
        output.push(HEX[(byte & 0x0f) as usize]);
    }
}

fn decode_hex(byte: u8) -> Result<u8, WifiControlError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(WifiControlError::InvalidCredential),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PSK_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    fn test_psk() -> WifiPsk {
        WifiPsk::from_hex(TEST_PSK_HEX).expect("valid test PSK")
    }

    #[test]
    fn typed_commands_have_bounded_exact_wire_encodings() {
        let ssid = WifiSsid::new(b"Aqua Lab".to_vec()).expect("valid SSID");
        assert_eq!(
            WifiControlRequest::SetSsid {
                network_id: 7,
                ssid: &ssid,
            }
            .encode()
            .expect("SSID command")
            .bytes(),
            b"SET_NETWORK 7 ssid 41717561204c6162"
        );
        let psk = test_psk();
        assert_eq!(
            WifiControlRequest::SetPsk {
                network_id: 7,
                psk: &psk,
            }
            .encode()
            .expect("PSK command")
            .bytes(),
            format!("SET_NETWORK 7 psk {TEST_PSK_HEX}").as_bytes()
        );
        assert_eq!(
            WifiControlRequest::SetWpa2Personal { network_id: 7 }
                .encode()
                .expect("WPA2 command")
                .bytes(),
            b"SET_NETWORK 7 key_mgmt WPA-PSK"
        );
        assert!(matches!(
            WifiControlRequest::RemoveNetwork { network_id: 4096 }.encode(),
            Err(WifiControlError::InvalidNetworkId)
        ));
    }

    #[test]
    fn responses_require_typed_bounded_authoritative_state() {
        assert_eq!(
            WifiControlRequest::AddNetwork.parse_response(b"7\n"),
            Ok(WifiControlResponse::NetworkAdded(7))
        );
        assert_eq!(
            WifiControlRequest::Status
                .parse_response(b"bssid=02:00:00:00:00:01\nid=7\nwpa_state=COMPLETED\n"),
            Ok(WifiControlResponse::Status(WifiControlStatus {
                state: WifiAssociationState::Completed,
                network_id: Some(7),
            }))
        );
        assert_eq!(
            WifiControlRequest::Status.parse_response(b"wpa_state=COMPLETED\n"),
            Err(WifiControlError::InvalidResponse)
        );
        assert_eq!(
            WifiControlRequest::Scan.parse_response(b"FAIL\n"),
            Err(WifiControlError::SupplicantRejected)
        );
        assert_eq!(
            WifiControlRequest::Scan
                .parse_response(&vec![b'x'; MAX_WIFI_CONTROL_RESPONSE_BYTES + 1]),
            Err(WifiControlError::ResponseTooLarge)
        );
    }

    #[test]
    fn credentials_store_only_a_derived_psk_and_parse_strictly() {
        let record = WifiCredentialRecord::new(
            WifiSsid::new(b"Aqua Lab".to_vec()).expect("valid SSID"),
            WifiSecurity::Wpa2Personal,
            test_psk(),
        );
        let encoded = record.encode();
        let encoded_text = std::str::from_utf8(encoded.bytes()).expect("UTF-8 record");
        assert!(!encoded_text.contains("correct horse"));
        assert!(encoded_text.contains("ssid=41717561204c6162"));
        assert!(encoded_text.contains(&format!("psk={TEST_PSK_HEX}")));
        let parsed = WifiCredentialRecord::parse(encoded.bytes()).expect("valid record");
        assert_eq!(parsed.ssid().bytes(), b"Aqua Lab");
        assert_eq!(parsed.security(), WifiSecurity::Wpa2Personal);
        assert_eq!(parsed.encode().bytes(), encoded.bytes());

        let malformed = encoded_text.replace("security=", "unknown=");
        assert!(matches!(
            WifiCredentialRecord::parse(malformed.as_bytes()),
            Err(WifiControlError::InvalidCredential)
        ));
    }

    #[test]
    fn secret_debug_output_is_redacted_and_passphrases_are_bounded() {
        let passphrase = WifiPassphrase::new("correct horse battery").expect("valid passphrase");
        assert_eq!(passphrase.with_bytes(|bytes| bytes.len()), 21);
        assert!(!format!("{passphrase:?}").contains("correct horse battery"));
        assert!(!format!("{:?}", test_psk()).contains(TEST_PSK_HEX));
        assert!(matches!(
            WifiPassphrase::new("short"),
            Err(WifiControlError::InvalidPassphrase)
        ));
        assert!(matches!(
            WifiPassphrase::new("contains\nnewline"),
            Err(WifiControlError::InvalidPassphrase)
        ));
    }

    #[test]
    fn credential_storage_requires_fixed_root_only_regular_files() {
        let safe = WifiCredentialMetadata {
            directory_uid: 0,
            directory_mode: 0o700,
            directory_is_symlink: false,
            file_uid: 0,
            file_mode: 0o600,
            file_is_regular: true,
            file_is_symlink: false,
            file_bytes: 180,
        };
        assert_eq!(validate_credential_metadata(safe), Ok(()));
        assert_eq!(
            validate_credential_metadata(WifiCredentialMetadata {
                file_mode: 0o640,
                ..safe
            }),
            Err(WifiControlError::UnsafeCredentialStorage)
        );
        assert_eq!(
            validate_credential_metadata(WifiCredentialMetadata {
                file_is_symlink: true,
                ..safe
            }),
            Err(WifiControlError::UnsafeCredentialStorage)
        );
    }
}
