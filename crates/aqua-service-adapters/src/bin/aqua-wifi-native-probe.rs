#[cfg(target_os = "linux")]
use aqua_service_adapters::wifi_control::{
    WifiAssociationState, WifiControlRequest, WifiControlResponse, WifiPassphrase, WifiSsid,
};
#[cfg(target_os = "linux")]
use aqua_service_adapters::{derive_wpa2_psk, WifiNativeControl};
#[cfg(target_os = "linux")]
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;
use std::process::ExitCode;

#[cfg(target_os = "linux")]
const EXPECTED_PSK: [u8; 32] = [
    0xf4, 0x2c, 0x6f, 0xc5, 0x2d, 0xf0, 0xeb, 0xef, 0x9e, 0xbb, 0x4b, 0x90, 0xb3, 0x8a, 0x5f, 0x90,
    0x2e, 0x83, 0xfe, 0x1b, 0x13, 0x5a, 0x70, 0xe2, 0x3a, 0xed, 0x76, 0x2e, 0x97, 0x10, 0xa1, 0x2e,
];

fn main() -> ExitCode {
    #[cfg(target_os = "linux")]
    let result = match std::env::args().nth(1).as_deref() {
        Some("native") => probe_native(),
        Some("broker") => probe_broker(),
        Some("broker-status") => probe_broker_request(
            b"AQUA-NETWORK/1 WIFI_STATUS wlan0\n",
            "AQUA-NETWORK/1 OK operation=wifi-status interface=wlan0",
        ),
        Some("broker-scan") => probe_broker_scan(),
        Some("broker-disconnect") => probe_broker_request(
            b"AQUA-NETWORK/1 WIFI_DISCONNECT wlan0\n",
            "AQUA-NETWORK/1 OK operation=wifi-disconnect interface=wlan0 authoritative=true",
        ),
        Some("broker-reconnect") => probe_broker_request(
            b"AQUA-NETWORK/1 WIFI_RECONNECT wlan0\n",
            "AQUA-NETWORK/1 OK operation=wifi-reconnect interface=wlan0 network_id=",
        ),
        Some("broker-forget") => probe_broker_request(
            b"AQUA-NETWORK/1 WIFI_FORGET wlan0\n",
            "AQUA-NETWORK/1 OK operation=wifi-forget interface=wlan0 authoritative=true credential_saved=false",
        ),
        _ => Err(
            "usage: aqua-wifi-native-probe native|broker|broker-status|broker-scan|broker-disconnect|broker-reconnect|broker-forget",
        ),
    };
    #[cfg(not(target_os = "linux"))]
    let result: Result<(), &'static str> = Err("Linux native Wi-Fi probe required");

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("[AQUA-NETWORK] stage=wifi-native-probe status=failed reason={reason}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "linux")]
fn probe_native() -> Result<(), &'static str> {
    let ssid = WifiSsid::new(b"IEEE".to_vec()).map_err(|_| "invalid-fixture-ssid")?;
    let passphrase = WifiPassphrase::new("password").map_err(|_| "invalid-fixture-passphrase")?;
    let psk = derive_wpa2_psk(&ssid, &passphrase).map_err(|_| "psk-derivation")?;
    if !psk.securely_matches(&EXPECTED_PSK) {
        return Err("psk-vector-mismatch");
    }
    let mut control = WifiNativeControl::connect().map_err(|_| "native-connect")?;
    let response = control
        .request(&WifiControlRequest::Status)
        .map_err(|_| "native-status")?;
    if response
        != WifiControlResponse::Status(aqua_service_adapters::wifi_control::WifiControlStatus {
            state: WifiAssociationState::Disconnected,
            network_id: None,
        })
    {
        return Err("unexpected-native-status");
    }
    println!(
        "[AQUA-NETWORK] stage=wifi-native-probe status=ok transport=libwpa_client psk_vector=true secrets_logged=false"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn probe_broker() -> Result<(), &'static str> {
    probe_broker_request(
        b"AQUA-NETWORK/1 WIFI_CONNECT wlan0 49454545 70617373776f7264\n",
        "AQUA-NETWORK/1 OK operation=wifi-connect interface=wlan0 network_id=",
    )?;
    println!(
        "[AQUA-NETWORK] stage=wifi-broker-probe status=ok peer_uid=1000 typed_connect=true credential_acknowledged=true"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn probe_broker_scan() -> Result<(), &'static str> {
    let response = broker_exchange(b"AQUA-NETWORK/1 WIFI_SCAN wlan0\n")?;
    if !response.starts_with(
        "AQUA-NETWORK/1 OK operation=wifi-scan interface=wlan0 count=1 authoritative=true",
    ) || !response.contains("network_0=49454545,")
        || !response.contains(",wpa2-personal")
    {
        return Err("broker-scan-response");
    }
    println!(
        "[AQUA-NETWORK] stage=wifi-broker-scan status=ok bounded=true count=1 security=wpa2-personal"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn probe_broker_request(request: &[u8], expected: &str) -> Result<(), &'static str> {
    let response = broker_exchange(request)?;
    if !response.starts_with(expected) {
        return Err("broker-response");
    }
    if (request
        .windows(b"WIFI_CONNECT".len())
        .any(|value| value == b"WIFI_CONNECT")
        || request
            .windows(b"WIFI_RECONNECT".len())
            .any(|value| value == b"WIFI_RECONNECT"))
        && !response.contains(" authoritative=true credential_saved=true")
    {
        return Err("broker-connect-acknowledgement");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn broker_exchange(request: &[u8]) -> Result<String, &'static str> {
    let mut stream =
        UnixStream::connect("/run/aqua-network/control.sock").map_err(|_| "broker-connect")?;
    stream.write_all(request).map_err(|_| "broker-write")?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|_| "broker-shutdown")?;
    let mut response = Vec::new();
    stream
        .take(513)
        .read_to_end(&mut response)
        .map_err(|_| "broker-read")?;
    String::from_utf8(response).map_err(|_| "broker-response-utf8")
}
