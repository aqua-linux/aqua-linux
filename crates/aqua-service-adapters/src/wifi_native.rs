use crate::wifi_control::MAX_WIFI_CONTROL_RESPONSE_BYTES;
#[cfg(target_os = "linux")]
use crate::wifi_control::{
    WifiControlRequest, WifiControlResponse, WifiPassphrase, WifiPsk, WifiSsid,
};
#[cfg(target_os = "linux")]
use std::ffi::c_void;
use std::fmt;
#[cfg(target_os = "linux")]
use std::marker::PhantomData;
#[cfg(target_os = "linux")]
use std::ptr::NonNull;
#[cfg(target_os = "linux")]
use std::rc::Rc;

const ABI_VERSION: u32 = 1;
#[cfg(target_os = "linux")]
pub const NATIVE_REQUEST_TIMEOUT_SECONDS: u8 = 10;

#[link(name = "aqua-wifi-native")]
#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn aqua_wifi_native_abi_version() -> u32;
    fn aqua_wifi_native_derive_wpa2_psk(
        ssid: *const u8,
        ssid_length: usize,
        passphrase: *const u8,
        passphrase_length: usize,
        out_psk: *mut u8,
    ) -> i32;
    fn aqua_wifi_native_open(out_handle: *mut *mut c_void) -> i32;
    fn aqua_wifi_native_close(handle: *mut c_void);
    fn aqua_wifi_native_request(
        handle: *mut c_void,
        command: *const u8,
        command_length: usize,
        response: *mut u8,
        response_length: *mut usize,
    ) -> i32;
}

#[cfg(target_os = "linux")]
pub struct WifiNativeControl {
    handle: NonNull<c_void>,
    not_send_or_sync: PhantomData<Rc<()>>,
}

#[cfg(target_os = "linux")]
impl fmt::Debug for WifiNativeControl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WifiNativeControl")
            .field("request_timeout_seconds", &NATIVE_REQUEST_TIMEOUT_SECONDS)
            .finish_non_exhaustive()
    }
}

#[cfg(target_os = "linux")]
impl WifiNativeControl {
    pub fn connect() -> Result<Self, WifiNativeError> {
        // SAFETY: This version query has no arguments or mutable global output.
        let actual = unsafe { aqua_wifi_native_abi_version() };
        validate_abi(actual)?;
        let mut raw_handle = std::ptr::null_mut();
        // SAFETY: `raw_handle` is a valid out pointer initialized to null.
        let status = unsafe { aqua_wifi_native_open(&mut raw_handle) };
        check_status(status)?;
        let handle = NonNull::new(raw_handle).ok_or(WifiNativeError::NullHandle)?;
        Ok(Self {
            handle,
            not_send_or_sync: PhantomData,
        })
    }

    pub fn request(
        &mut self,
        request: &WifiControlRequest<'_>,
    ) -> Result<WifiControlResponse, WifiNativeError> {
        let command = request.encode().map_err(WifiNativeError::Control)?;
        let mut response = [0; MAX_WIFI_CONTROL_RESPONSE_BYTES];
        let mut response_length = response.len();
        // SAFETY: The handle is live, command and response buffers are valid for
        // their declared lengths, and the native ABI caps the returned length.
        let status = unsafe {
            aqua_wifi_native_request(
                self.handle.as_ptr(),
                command.bytes().as_ptr(),
                command.bytes().len(),
                response.as_mut_ptr(),
                &mut response_length,
            )
        };
        check_status(status)?;
        let response = bounded_response(&response, response_length)?;
        request
            .parse_response(response)
            .map_err(WifiNativeError::Control)
    }
}

#[cfg(target_os = "linux")]
impl Drop for WifiNativeControl {
    fn drop(&mut self) {
        // SAFETY: The handle is uniquely owned and closed exactly once here.
        unsafe { aqua_wifi_native_close(self.handle.as_ptr()) };
    }
}

#[cfg(target_os = "linux")]
pub fn derive_wpa2_psk(
    ssid: &WifiSsid,
    passphrase: &WifiPassphrase,
) -> Result<WifiPsk, WifiNativeError> {
    validate_abi(unsafe { aqua_wifi_native_abi_version() })?;
    let mut psk = [0; 32];
    let status = passphrase.with_bytes(|passphrase| {
        // SAFETY: All input slices and the fixed-size output are valid for the
        // duration of this bounded synchronous derivation call.
        unsafe {
            aqua_wifi_native_derive_wpa2_psk(
                ssid.bytes().as_ptr(),
                ssid.bytes().len(),
                passphrase.as_ptr(),
                passphrase.len(),
                psk.as_mut_ptr(),
            )
        }
    });
    check_status(status)?;
    Ok(WifiPsk::from_bytes(psk))
}

fn validate_abi(actual: u32) -> Result<(), WifiNativeError> {
    if actual == ABI_VERSION {
        Ok(())
    } else {
        Err(WifiNativeError::AbiVersion {
            expected: ABI_VERSION,
            actual,
        })
    }
}

fn bounded_response(response: &[u8], length: usize) -> Result<&[u8], WifiNativeError> {
    if length == 0 || length > response.len() || length > MAX_WIFI_CONTROL_RESPONSE_BYTES {
        return Err(WifiNativeError::InvalidResponseLength(length));
    }
    Ok(&response[..length])
}

fn check_status(status: i32) -> Result<(), WifiNativeError> {
    match status {
        0 => Ok(()),
        -1 => Err(WifiNativeError::InvalidArgument),
        -2 => Err(WifiNativeError::ConnectFailed),
        -3 => Err(WifiNativeError::Timeout),
        -4 => Err(WifiNativeError::ApiFailed),
        -5 => Err(WifiNativeError::BoundsExceeded),
        -6 => Err(WifiNativeError::DerivationFailed),
        value => Err(WifiNativeError::UnknownStatus(value)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiNativeError {
    AbiVersion {
        expected: u32,
        actual: u32,
    },
    #[cfg(target_os = "linux")]
    NullHandle,
    InvalidArgument,
    ConnectFailed,
    Timeout,
    ApiFailed,
    BoundsExceeded,
    DerivationFailed,
    InvalidResponseLength(usize),
    UnknownStatus(i32),
    #[cfg(target_os = "linux")]
    Control(crate::wifi_control::WifiControlError),
}

impl fmt::Display for WifiNativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbiVersion { expected, actual } => {
                write!(
                    formatter,
                    "Wi-Fi native ABI mismatch: expected {expected}, got {actual}"
                )
            }
            #[cfg(target_os = "linux")]
            Self::NullHandle => formatter.write_str("Wi-Fi native API returned a null handle"),
            Self::InvalidArgument => formatter.write_str("invalid Wi-Fi native argument"),
            Self::ConnectFailed => formatter.write_str("wpa_supplicant control is unavailable"),
            Self::Timeout => formatter.write_str("wpa_supplicant control request timed out"),
            Self::ApiFailed => formatter.write_str("wpa_supplicant control request failed"),
            Self::BoundsExceeded => formatter.write_str("Wi-Fi native bound exceeded"),
            Self::DerivationFailed => formatter.write_str("WPA2 PSK derivation failed"),
            Self::InvalidResponseLength(length) => {
                write!(formatter, "invalid Wi-Fi native response length: {length}")
            }
            Self::UnknownStatus(status) => {
                write!(formatter, "unknown Wi-Fi native status: {status}")
            }
            #[cfg(target_os = "linux")]
            Self::Control(error) => write!(formatter, "invalid Wi-Fi control response: {error}"),
        }
    }
}

impl std::error::Error for WifiNativeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_abi_and_status_values_fail_closed() {
        assert_eq!(validate_abi(1), Ok(()));
        assert_eq!(
            validate_abi(2),
            Err(WifiNativeError::AbiVersion {
                expected: 1,
                actual: 2,
            })
        );
        assert_eq!(check_status(-3), Err(WifiNativeError::Timeout));
        assert_eq!(check_status(42), Err(WifiNativeError::UnknownStatus(42)));
    }

    #[test]
    fn native_response_length_is_strictly_bounded() {
        let response = [0_u8; 8];
        assert_eq!(bounded_response(&response, 4), Ok(&response[..4]));
        assert_eq!(
            bounded_response(&response, 0),
            Err(WifiNativeError::InvalidResponseLength(0))
        );
        assert_eq!(
            bounded_response(&response, 9),
            Err(WifiNativeError::InvalidResponseLength(9))
        );
    }
}
