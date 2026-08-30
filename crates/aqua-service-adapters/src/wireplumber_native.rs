#[cfg(target_os = "linux")]
use crate::PipeWireApi;
use crate::{AudioDeviceKind, PipeWireApiNode, PipeWireApiPhase, PipeWireApiSnapshot};
use std::ffi::{c_char, CStr};
#[cfg(target_os = "linux")]
use std::ffi::{c_void, CString};
use std::fmt;
#[cfg(target_os = "linux")]
use std::marker::PhantomData;
#[cfg(target_os = "linux")]
use std::ptr::NonNull;
#[cfg(target_os = "linux")]
use std::rc::Rc;
use std::time::Duration;

const ABI_VERSION: u32 = 1;
const MAX_NODES: usize = 32;
const NODE_NAME_BYTES: usize = 65;
const NODE_DESCRIPTION_BYTES: usize = 97;

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeNode {
    name: [c_char; NODE_NAME_BYTES],
    description: [c_char; NODE_DESCRIPTION_BYTES],
    kind: u8,
    volume_percent: u8,
    muted: u8,
    reserved: u8,
}

impl Default for NativeNode {
    fn default() -> Self {
        Self {
            name: [0; NODE_NAME_BYTES],
            description: [0; NODE_DESCRIPTION_BYTES],
            kind: 0,
            volume_percent: 0,
            muted: 0,
            reserved: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeSnapshot {
    abi_version: u32,
    phase: u32,
    generation: u64,
    node_count: u32,
    reserved: u32,
    default_output: [c_char; NODE_NAME_BYTES],
    default_input: [c_char; NODE_NAME_BYTES],
    nodes: [NativeNode; MAX_NODES],
}

impl Default for NativeSnapshot {
    fn default() -> Self {
        Self {
            abi_version: 0,
            phase: 0,
            generation: 0,
            node_count: 0,
            reserved: 0,
            default_output: [0; NODE_NAME_BYTES],
            default_input: [0; NODE_NAME_BYTES],
            nodes: [NativeNode::default(); MAX_NODES],
        }
    }
}

#[link(name = "aqua-audio-native")]
#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn aqua_audio_native_abi_version() -> u32;
    fn aqua_audio_native_open(timeout_ms: u32, handle: *mut *mut c_void) -> i32;
    fn aqua_audio_native_close(handle: *mut c_void);
    fn aqua_audio_native_last_error(handle: *mut c_void) -> *const c_char;
    fn aqua_audio_native_snapshot(
        handle: *mut c_void,
        timeout_ms: u32,
        snapshot: *mut NativeSnapshot,
    ) -> i32;
    fn aqua_audio_native_set_output_volume(
        handle: *mut c_void,
        node_name: *const c_char,
        volume_percent: u8,
        timeout_ms: u32,
    ) -> i32;
    fn aqua_audio_native_set_output_muted(
        handle: *mut c_void,
        node_name: *const c_char,
        muted: u8,
        timeout_ms: u32,
    ) -> i32;
    fn aqua_audio_native_set_configured_default_output(
        handle: *mut c_void,
        node_name: *const c_char,
        timeout_ms: u32,
    ) -> i32;
}

#[cfg(target_os = "linux")]
pub struct WirePlumberNativeApi {
    handle: NonNull<c_void>,
    timeout_ms: u32,
    not_send_or_sync: PhantomData<Rc<()>>,
}

#[cfg(target_os = "linux")]
impl fmt::Debug for WirePlumberNativeApi {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WirePlumberNativeApi")
            .field("timeout_ms", &self.timeout_ms)
            .finish_non_exhaustive()
    }
}

#[cfg(target_os = "linux")]
impl WirePlumberNativeApi {
    pub fn connect(timeout: Duration) -> Result<Self, WirePlumberNativeError> {
        let timeout_ms = bounded_timeout_ms(timeout)?;
        // SAFETY: This version query has no arguments or mutable global output.
        let actual_abi = unsafe { aqua_audio_native_abi_version() };
        if actual_abi != ABI_VERSION {
            return Err(WirePlumberNativeError::AbiVersion {
                expected: ABI_VERSION,
                actual: actual_abi,
            });
        }

        let mut raw_handle = std::ptr::null_mut();
        // SAFETY: `raw_handle` is a valid out pointer and the timeout is bounded.
        let status = unsafe { aqua_audio_native_open(timeout_ms, &mut raw_handle) };
        let handle = NonNull::new(raw_handle);
        if status != 0 {
            let error = native_error(handle, status);
            if let Some(handle) = handle {
                // SAFETY: A failed open may return an owned handle for diagnostics.
                unsafe { aqua_audio_native_close(handle.as_ptr()) };
            }
            return Err(error);
        }
        let handle = handle.ok_or(WirePlumberNativeError::NullHandle)?;
        Ok(Self {
            handle,
            timeout_ms,
            not_send_or_sync: PhantomData,
        })
    }

    pub const fn timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    fn node_name(name: &str) -> Result<CString, WirePlumberNativeError> {
        if name.len() >= NODE_NAME_BYTES {
            return Err(WirePlumberNativeError::NodeNameTooLong(name.len()));
        }
        CString::new(name).map_err(|_| WirePlumberNativeError::InteriorNul)
    }

    fn call_status(&self, status: i32) -> Result<(), WirePlumberNativeError> {
        if status == 0 {
            Ok(())
        } else {
            Err(native_error(Some(self.handle), status))
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for WirePlumberNativeApi {
    fn drop(&mut self) {
        // SAFETY: `handle` is uniquely owned and closed exactly once here.
        unsafe { aqua_audio_native_close(self.handle.as_ptr()) };
    }
}

#[cfg(target_os = "linux")]
impl PipeWireApi for WirePlumberNativeApi {
    type Error = WirePlumberNativeError;

    fn synchronized_snapshot(&mut self) -> Result<PipeWireApiSnapshot, Self::Error> {
        let mut snapshot = NativeSnapshot::default();
        // SAFETY: The handle is live and `snapshot` is a correctly sized out value.
        let status = unsafe {
            aqua_audio_native_snapshot(self.handle.as_ptr(), self.timeout_ms, &mut snapshot)
        };
        self.call_status(status)?;
        decode_snapshot(&snapshot)
    }

    fn set_output_volume(
        &mut self,
        node_name: &str,
        volume_percent: u8,
    ) -> Result<(), Self::Error> {
        if volume_percent > 100 {
            return Err(WirePlumberNativeError::InvalidVolume(volume_percent));
        }
        let node_name = Self::node_name(node_name)?;
        // SAFETY: The handle is live and the node name is a bounded C string.
        let status = unsafe {
            aqua_audio_native_set_output_volume(
                self.handle.as_ptr(),
                node_name.as_ptr(),
                volume_percent,
                self.timeout_ms,
            )
        };
        self.call_status(status)
    }

    fn set_output_muted(&mut self, node_name: &str, muted: bool) -> Result<(), Self::Error> {
        let node_name = Self::node_name(node_name)?;
        // SAFETY: The handle is live and the node name is a bounded C string.
        let status = unsafe {
            aqua_audio_native_set_output_muted(
                self.handle.as_ptr(),
                node_name.as_ptr(),
                u8::from(muted),
                self.timeout_ms,
            )
        };
        self.call_status(status)
    }

    fn set_configured_default_output(&mut self, node_name: &str) -> Result<(), Self::Error> {
        let node_name = Self::node_name(node_name)?;
        // SAFETY: The handle is live and the node name is a bounded C string.
        let status = unsafe {
            aqua_audio_native_set_configured_default_output(
                self.handle.as_ptr(),
                node_name.as_ptr(),
                self.timeout_ms,
            )
        };
        self.call_status(status)
    }
}

fn decode_snapshot(
    snapshot: &NativeSnapshot,
) -> Result<PipeWireApiSnapshot, WirePlumberNativeError> {
    if snapshot.abi_version != ABI_VERSION {
        return Err(WirePlumberNativeError::AbiVersion {
            expected: ABI_VERSION,
            actual: snapshot.abi_version,
        });
    }
    if snapshot.reserved != 0 {
        return Err(WirePlumberNativeError::ReservedField);
    }
    let node_count = usize::try_from(snapshot.node_count)
        .map_err(|_| WirePlumberNativeError::NodeCount(snapshot.node_count))?;
    if node_count > MAX_NODES {
        return Err(WirePlumberNativeError::NodeCount(snapshot.node_count));
    }
    let phase = match snapshot.phase {
        0 => PipeWireApiPhase::Disconnected,
        1 => PipeWireApiPhase::Connecting,
        2 => PipeWireApiPhase::Synchronizing,
        3 => PipeWireApiPhase::Ready,
        4 => PipeWireApiPhase::Degraded,
        value => return Err(WirePlumberNativeError::InvalidPhase(value)),
    };
    let nodes = snapshot.nodes[..node_count]
        .iter()
        .map(|node| {
            if node.reserved != 0 {
                return Err(WirePlumberNativeError::ReservedField);
            }
            let kind = match node.kind {
                0 => AudioDeviceKind::Output,
                1 => AudioDeviceKind::Input,
                value => return Err(WirePlumberNativeError::InvalidNodeKind(value)),
            };
            if node.muted > 1 {
                return Err(WirePlumberNativeError::InvalidBoolean(node.muted));
            }
            PipeWireApiNode::new(
                decode_required_text(&node.name)?,
                decode_required_text(&node.description)?,
                kind,
                node.volume_percent,
                node.muted == 1,
            )
            .map_err(|error| WirePlumberNativeError::InvalidSnapshot(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let default_output = decode_optional_text(&snapshot.default_output)?;
    let default_input = decode_optional_text(&snapshot.default_input)?;
    PipeWireApiSnapshot::new(
        snapshot.generation,
        phase,
        nodes,
        default_output,
        default_input,
    )
    .map_err(|error| WirePlumberNativeError::InvalidSnapshot(error.to_string()))
}

fn decode_required_text<const N: usize>(
    bytes: &[c_char; N],
) -> Result<String, WirePlumberNativeError> {
    decode_optional_text(bytes)?.ok_or(WirePlumberNativeError::EmptyText)
}

fn decode_optional_text<const N: usize>(
    bytes: &[c_char; N],
) -> Result<Option<String>, WirePlumberNativeError> {
    let terminator = bytes
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(WirePlumberNativeError::UnterminatedText)?;
    if terminator == 0 {
        return Ok(None);
    }
    let raw = &bytes[..=terminator];
    // SAFETY: The slice contains a verified trailing NUL and stays alive for the call.
    let text = unsafe { CStr::from_ptr(raw.as_ptr()) }
        .to_str()
        .map_err(|_| WirePlumberNativeError::InvalidUtf8)?;
    Ok(Some(text.to_string()))
}

fn bounded_timeout_ms(timeout: Duration) -> Result<u32, WirePlumberNativeError> {
    let milliseconds = timeout.as_millis();
    if milliseconds == 0 || milliseconds > u128::from(u32::MAX) {
        return Err(WirePlumberNativeError::InvalidTimeout);
    }
    Ok(milliseconds as u32)
}

#[cfg(target_os = "linux")]
fn native_error(handle: Option<NonNull<c_void>>, status: i32) -> WirePlumberNativeError {
    let message = handle
        .and_then(|handle| {
            // SAFETY: The library owns this NUL-terminated diagnostic for the handle lifetime.
            let pointer = unsafe { aqua_audio_native_last_error(handle.as_ptr()) };
            if pointer.is_null() {
                None
            } else {
                // SAFETY: A non-null diagnostic pointer is NUL-terminated by the C ABI.
                unsafe { CStr::from_ptr(pointer) }
                    .to_str()
                    .ok()
                    .map(str::to_owned)
            }
        })
        .filter(|message| !message.is_empty())
        .unwrap_or_else(|| "native audio operation failed".to_string());
    WirePlumberNativeError::Native { status, message }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub enum WirePlumberNativeError {
    AbiVersion { expected: u32, actual: u32 },
    NullHandle,
    Native { status: i32, message: String },
    InvalidTimeout,
    NodeNameTooLong(usize),
    InteriorNul,
    NodeCount(u32),
    InvalidPhase(u32),
    InvalidNodeKind(u8),
    InvalidBoolean(u8),
    ReservedField,
    EmptyText,
    UnterminatedText,
    InvalidUtf8,
    InvalidVolume(u8),
    InvalidSnapshot(String),
}

impl fmt::Display for WirePlumberNativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbiVersion { expected, actual } => {
                write!(
                    formatter,
                    "native audio ABI mismatch: expected {expected}, got {actual}"
                )
            }
            Self::NullHandle => formatter.write_str("native audio API returned a null handle"),
            Self::Native { status, message } => {
                write!(formatter, "native audio error {status}: {message}")
            }
            Self::InvalidTimeout => formatter.write_str("native audio timeout is out of range"),
            Self::NodeNameTooLong(length) => {
                write!(
                    formatter,
                    "native audio node name is too long: {length} bytes"
                )
            }
            Self::InteriorNul => formatter.write_str("native audio node name contains NUL"),
            Self::NodeCount(count) => write!(formatter, "invalid native node count: {count}"),
            Self::InvalidPhase(value) => write!(formatter, "invalid native phase: {value}"),
            Self::InvalidNodeKind(value) => write!(formatter, "invalid native node kind: {value}"),
            Self::InvalidBoolean(value) => write!(formatter, "invalid native boolean: {value}"),
            Self::ReservedField => formatter.write_str("native reserved field is non-zero"),
            Self::EmptyText => formatter.write_str("native required text is empty"),
            Self::UnterminatedText => formatter.write_str("native text is not NUL-terminated"),
            Self::InvalidUtf8 => formatter.write_str("native text is not valid UTF-8"),
            Self::InvalidVolume(value) => write!(formatter, "invalid native volume: {value}"),
            Self::InvalidSnapshot(error) => write!(formatter, "invalid native snapshot: {error}"),
        }
    }
}

impl std::error::Error for WirePlumberNativeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_text<const N: usize>(destination: &mut [c_char; N], text: &str) {
        for (slot, byte) in destination.iter_mut().zip(text.bytes()) {
            *slot = byte as c_char;
        }
    }

    #[test]
    fn decodes_bounded_ready_snapshot() {
        let mut snapshot = NativeSnapshot {
            abi_version: ABI_VERSION,
            phase: 3,
            generation: 9,
            node_count: 2,
            ..NativeSnapshot::default()
        };
        write_text(&mut snapshot.default_output, "sink.one");
        write_text(&mut snapshot.default_input, "source.one");
        write_text(&mut snapshot.nodes[0].name, "sink.one");
        write_text(&mut snapshot.nodes[0].description, "Built-in Output");
        snapshot.nodes[0].volume_percent = 72;
        snapshot.nodes[0].muted = 1;
        write_text(&mut snapshot.nodes[1].name, "source.one");
        write_text(&mut snapshot.nodes[1].description, "Built-in Input");
        snapshot.nodes[1].kind = 1;

        let decoded = decode_snapshot(&snapshot).unwrap();
        assert_eq!(decoded.generation(), 9);
        assert_eq!(decoded.default_output(), Some("sink.one"));
        assert_eq!(decoded.default_input(), Some("source.one"));
        assert_eq!(decoded.nodes()[0].volume_percent(), 72);
        assert!(decoded.nodes()[0].muted());
    }

    #[test]
    fn rust_layout_matches_native_abi_version_one() {
        assert_eq!(std::mem::size_of::<NativeNode>(), 166);
        assert_eq!(std::mem::offset_of!(NativeSnapshot, nodes), 154);
        assert_eq!(std::mem::size_of::<NativeSnapshot>(), 5_472);
    }

    #[test]
    fn rejects_abi_bounds_and_unterminated_text() {
        let wrong_abi = NativeSnapshot {
            abi_version: 2,
            ..NativeSnapshot::default()
        };
        assert!(matches!(
            decode_snapshot(&wrong_abi),
            Err(WirePlumberNativeError::AbiVersion { .. })
        ));

        let too_many = NativeSnapshot {
            abi_version: ABI_VERSION,
            node_count: 33,
            ..NativeSnapshot::default()
        };
        assert_eq!(
            decode_snapshot(&too_many),
            Err(WirePlumberNativeError::NodeCount(33))
        );

        let reserved = NativeSnapshot {
            abi_version: ABI_VERSION,
            reserved: 1,
            ..NativeSnapshot::default()
        };
        assert_eq!(
            decode_snapshot(&reserved),
            Err(WirePlumberNativeError::ReservedField)
        );

        let mut unterminated = NativeSnapshot {
            abi_version: ABI_VERSION,
            phase: 3,
            node_count: 1,
            ..NativeSnapshot::default()
        };
        unterminated.nodes[0].name.fill(b'x' as c_char);
        write_text(&mut unterminated.nodes[0].description, "Output");
        assert_eq!(
            decode_snapshot(&unterminated),
            Err(WirePlumberNativeError::UnterminatedText)
        );
    }

    #[test]
    fn timeout_inputs_are_bounded() {
        assert_eq!(
            bounded_timeout_ms(Duration::ZERO),
            Err(WirePlumberNativeError::InvalidTimeout)
        );
        assert_eq!(bounded_timeout_ms(Duration::from_secs(2)).unwrap(), 2_000);
    }
}
