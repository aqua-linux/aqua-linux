use std::collections::HashSet;
use std::fmt;

mod pipewire;

pub use pipewire::{
    PipeWireApi, PipeWireApiNode, PipeWireApiPhase, PipeWireApiSnapshot, PipeWireApiTransport,
    PipeWireTransportError,
};

pub const MAX_AUDIO_DEVICES: usize = 32;
pub const MAX_AUDIO_DEVICE_ID_BYTES: usize = 64;
pub const MAX_AUDIO_DEVICE_NAME_BYTES: usize = 96;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioServiceHealth {
    Unavailable,
    Starting,
    Ready,
    Degraded,
}

impl AudioServiceHealth {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDeviceKind {
    Output,
    Input,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    id: String,
    name: String,
    kind: AudioDeviceKind,
}

impl AudioDevice {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: AudioDeviceKind,
    ) -> Result<Self, AudioAdapterError> {
        let device = Self {
            id: id.into(),
            name: name.into(),
            kind,
        };
        validate_device(&device)?;
        Ok(device)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn kind(&self) -> AudioDeviceKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioAuthoritativeState {
    generation: u64,
    health: AudioServiceHealth,
    devices: Vec<AudioDevice>,
    default_output: Option<String>,
    default_input: Option<String>,
    output_volume_percent: u8,
    output_muted: bool,
}

impl AudioAuthoritativeState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        generation: u64,
        health: AudioServiceHealth,
        devices: Vec<AudioDevice>,
        default_output: Option<String>,
        default_input: Option<String>,
        output_volume_percent: u8,
        output_muted: bool,
    ) -> Result<Self, AudioAdapterError> {
        let state = Self {
            generation,
            health,
            devices,
            default_output,
            default_input,
            output_volume_percent,
            output_muted,
        };
        state.validate()?;
        Ok(state)
    }

    pub fn unavailable(
        generation: u64,
        health: AudioServiceHealth,
    ) -> Result<Self, AudioAdapterError> {
        if health == AudioServiceHealth::Ready {
            return Err(AudioAdapterError::ReadyRequiresAuthoritativeState);
        }
        Ok(Self {
            generation,
            health,
            devices: Vec::new(),
            default_output: None,
            default_input: None,
            output_volume_percent: 0,
            output_muted: true,
        })
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn health(&self) -> AudioServiceHealth {
        self.health
    }

    pub fn devices(&self) -> &[AudioDevice] {
        &self.devices
    }

    pub fn default_output(&self) -> Option<&str> {
        self.default_output.as_deref()
    }

    pub fn default_input(&self) -> Option<&str> {
        self.default_input.as_deref()
    }

    pub const fn output_volume_percent(&self) -> u8 {
        self.output_volume_percent
    }

    pub const fn output_muted(&self) -> bool {
        self.output_muted
    }

    pub fn output_device(&self) -> Option<&AudioDevice> {
        let route = self.default_output.as_deref()?;
        self.devices.iter().find(|device| device.id == route)
    }

    pub fn input_device(&self) -> Option<&AudioDevice> {
        let route = self.default_input.as_deref()?;
        self.devices.iter().find(|device| device.id == route)
    }

    pub fn controls_enabled(&self) -> bool {
        self.health == AudioServiceHealth::Ready && self.output_device().is_some()
    }

    fn validate(&self) -> Result<(), AudioAdapterError> {
        if self.output_volume_percent > 100 {
            return Err(AudioAdapterError::InvalidVolume(self.output_volume_percent));
        }
        if self.devices.len() > MAX_AUDIO_DEVICES {
            return Err(AudioAdapterError::TooManyDevices(self.devices.len()));
        }
        let mut ids = HashSet::with_capacity(self.devices.len());
        for device in &self.devices {
            validate_device(device)?;
            if !ids.insert(device.id.as_str()) {
                return Err(AudioAdapterError::DuplicateDeviceId(device.id.clone()));
            }
        }
        validate_route(
            "output",
            self.default_output.as_deref(),
            AudioDeviceKind::Output,
            &self.devices,
        )?;
        validate_route(
            "input",
            self.default_input.as_deref(),
            AudioDeviceKind::Input,
            &self.devices,
        )?;
        if self.health != AudioServiceHealth::Ready
            && (self.default_output.is_some() || self.default_input.is_some())
        {
            return Err(AudioAdapterError::RouteWhileServiceUnavailable);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioIntent {
    SetOutputVolume(u8),
    SetOutputMuted(bool),
    SetDefaultOutput(String),
}

impl AudioIntent {
    fn is_confirmed_by(&self, state: &AudioAuthoritativeState) -> bool {
        match self {
            Self::SetOutputVolume(value) => state.output_volume_percent == *value,
            Self::SetOutputMuted(value) => state.output_muted == *value,
            Self::SetDefaultOutput(id) => state.default_output.as_deref() == Some(id.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioRequest {
    id: u64,
    expected_generation: u64,
    intent: AudioIntent,
}

impl AudioRequest {
    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn expected_generation(&self) -> u64 {
        self.expected_generation
    }

    pub const fn intent(&self) -> &AudioIntent {
        &self.intent
    }
}

pub trait AudioBackend {
    type Error;

    fn authoritative_state(&mut self) -> Result<AudioAuthoritativeState, Self::Error>;
    fn submit(&mut self, request: &AudioRequest) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBackendDriveOutcome {
    pub reconcile: AudioReconcileOutcome,
    pub submitted_request_id: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AudioBackendDriveError<E> {
    Adapter(AudioAdapterError),
    Backend(E),
}

impl<E: fmt::Display> fmt::Display for AudioBackendDriveError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(error) => write!(formatter, "audio adapter error: {error}"),
            Self::Backend(error) => write!(formatter, "audio backend error: {error}"),
        }
    }
}

impl<E> std::error::Error for AudioBackendDriveError<E> where E: std::error::Error + 'static {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioReconcileOutcome {
    pub generation_advanced: bool,
    pub request_confirmed: bool,
    pub request_rejected_or_lost: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioServiceAdapter {
    state: AudioAuthoritativeState,
    desired_volume_percent: u8,
    desired_muted: bool,
    desired_output: Option<String>,
    pending_request: Option<AudioRequest>,
    next_request_id: u64,
}

impl Default for AudioServiceAdapter {
    fn default() -> Self {
        Self::with_preferences(70, false).expect("default audio preferences are bounded")
    }
}

impl AudioServiceAdapter {
    pub fn with_preferences(
        desired_volume_percent: u8,
        desired_muted: bool,
    ) -> Result<Self, AudioAdapterError> {
        if desired_volume_percent > 100 {
            return Err(AudioAdapterError::InvalidVolume(desired_volume_percent));
        }
        Ok(Self {
            state: AudioAuthoritativeState::unavailable(0, AudioServiceHealth::Unavailable)
                .expect("unavailable bootstrap state is valid"),
            desired_volume_percent,
            desired_muted,
            desired_output: None,
            pending_request: None,
            next_request_id: 1,
        })
    }

    pub const fn state(&self) -> &AudioAuthoritativeState {
        &self.state
    }

    pub const fn desired_volume_percent(&self) -> u8 {
        self.desired_volume_percent
    }

    pub const fn desired_muted(&self) -> bool {
        self.desired_muted
    }

    pub fn desired_output(&self) -> Option<&str> {
        self.desired_output.as_deref()
    }

    pub const fn pending_request(&self) -> Option<&AudioRequest> {
        self.pending_request.as_ref()
    }

    pub fn controls_enabled(&self) -> bool {
        self.state.controls_enabled()
    }

    pub fn backend_applied(&self) -> bool {
        self.controls_enabled()
            && self.pending_request.is_none()
            && self.state.output_volume_percent == self.desired_volume_percent
            && self.state.output_muted == self.desired_muted
            && self
                .desired_output
                .as_deref()
                .is_none_or(|id| self.state.default_output.as_deref() == Some(id))
    }

    pub fn set_desired_volume(&mut self, volume_percent: u8) -> Result<bool, AudioAdapterError> {
        if volume_percent > 100 {
            return Err(AudioAdapterError::InvalidVolume(volume_percent));
        }
        if self.desired_volume_percent == volume_percent {
            return Ok(false);
        }
        self.desired_volume_percent = volume_percent;
        Ok(true)
    }

    pub fn set_desired_muted(&mut self, muted: bool) -> bool {
        if self.desired_muted == muted {
            return false;
        }
        self.desired_muted = muted;
        true
    }

    pub fn set_desired_output(&mut self, id: impl Into<String>) -> Result<bool, AudioAdapterError> {
        let id = id.into();
        validate_id(&id)?;
        if self.desired_output.as_deref() == Some(id.as_str()) {
            return Ok(false);
        }
        self.desired_output = Some(id);
        Ok(true)
    }

    pub fn reconcile(
        &mut self,
        next: AudioAuthoritativeState,
    ) -> Result<AudioReconcileOutcome, AudioAdapterError> {
        next.validate()?;
        if next.generation < self.state.generation {
            return Err(AudioAdapterError::StaleGeneration {
                current: self.state.generation,
                received: next.generation,
            });
        }
        if next.generation == self.state.generation {
            if next == self.state {
                return Ok(AudioReconcileOutcome {
                    generation_advanced: false,
                    request_confirmed: false,
                    request_rejected_or_lost: false,
                });
            }
            return Err(AudioAdapterError::ConflictingGeneration(next.generation));
        }

        let mut request_confirmed = false;
        let mut request_rejected_or_lost = false;
        if let Some(request) = self.pending_request.take() {
            request_confirmed =
                next.health == AudioServiceHealth::Ready && request.intent.is_confirmed_by(&next);
            request_rejected_or_lost = !request_confirmed;
        }
        self.state = next;
        Ok(AudioReconcileOutcome {
            generation_advanced: true,
            request_confirmed,
            request_rejected_or_lost,
        })
    }

    pub fn next_reconciliation_request(
        &mut self,
    ) -> Result<Option<AudioRequest>, AudioAdapterError> {
        if self.pending_request.is_some() || !self.controls_enabled() {
            return Ok(None);
        }
        let intent = if self.state.output_volume_percent != self.desired_volume_percent {
            Some(AudioIntent::SetOutputVolume(self.desired_volume_percent))
        } else if self.state.output_muted != self.desired_muted {
            Some(AudioIntent::SetOutputMuted(self.desired_muted))
        } else if let Some(id) = self.desired_output.as_deref() {
            if self.state.default_output.as_deref() != Some(id) {
                let device = self
                    .state
                    .devices
                    .iter()
                    .find(|device| device.id == id)
                    .ok_or_else(|| AudioAdapterError::UnknownDesiredOutput(id.to_string()))?;
                if device.kind != AudioDeviceKind::Output {
                    return Err(AudioAdapterError::WrongRouteKind {
                        route: "output",
                        id: id.to_string(),
                    });
                }
                Some(AudioIntent::SetDefaultOutput(id.to_string()))
            } else {
                None
            }
        } else {
            None
        };
        let Some(intent) = intent else {
            return Ok(None);
        };
        let request = AudioRequest {
            id: self.next_request_id,
            expected_generation: self.state.generation,
            intent,
        };
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(AudioAdapterError::RequestIdExhausted)?;
        self.pending_request = Some(request.clone());
        Ok(Some(request))
    }

    pub fn drive_backend_once<B: AudioBackend>(
        &mut self,
        backend: &mut B,
    ) -> Result<AudioBackendDriveOutcome, AudioBackendDriveError<B::Error>> {
        let next = backend
            .authoritative_state()
            .map_err(AudioBackendDriveError::Backend)?;
        let reconcile = self
            .reconcile(next)
            .map_err(AudioBackendDriveError::Adapter)?;
        let Some(request) = self
            .next_reconciliation_request()
            .map_err(AudioBackendDriveError::Adapter)?
        else {
            return Ok(AudioBackendDriveOutcome {
                reconcile,
                submitted_request_id: None,
            });
        };
        if let Err(error) = backend.submit(&request) {
            self.cancel_pending_request(request.id);
            return Err(AudioBackendDriveError::Backend(error));
        }
        Ok(AudioBackendDriveOutcome {
            reconcile,
            submitted_request_id: Some(request.id),
        })
    }

    fn cancel_pending_request(&mut self, request_id: u64) {
        if self.pending_request.as_ref().map(AudioRequest::id) == Some(request_id) {
            self.pending_request = None;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioAdapterError {
    InvalidVolume(u8),
    TooManyDevices(usize),
    InvalidDeviceId,
    InvalidDeviceName,
    DuplicateDeviceId(String),
    UnknownRoute { route: &'static str, id: String },
    WrongRouteKind { route: &'static str, id: String },
    RouteWhileServiceUnavailable,
    ReadyRequiresAuthoritativeState,
    StaleGeneration { current: u64, received: u64 },
    ConflictingGeneration(u64),
    UnknownDesiredOutput(String),
    RequestIdExhausted,
}

impl fmt::Display for AudioAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AudioAdapterError {}

fn validate_device(device: &AudioDevice) -> Result<(), AudioAdapterError> {
    validate_id(&device.id)?;
    if device.name.is_empty()
        || device.name.len() > MAX_AUDIO_DEVICE_NAME_BYTES
        || device.name.chars().any(char::is_control)
    {
        return Err(AudioAdapterError::InvalidDeviceName);
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), AudioAdapterError> {
    if id.is_empty()
        || id.len() > MAX_AUDIO_DEVICE_ID_BYTES
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(AudioAdapterError::InvalidDeviceId);
    }
    Ok(())
}

fn validate_route(
    route: &'static str,
    id: Option<&str>,
    expected_kind: AudioDeviceKind,
    devices: &[AudioDevice],
) -> Result<(), AudioAdapterError> {
    let Some(id) = id else {
        return Ok(());
    };
    let device = devices
        .iter()
        .find(|device| device.id == id)
        .ok_or_else(|| AudioAdapterError::UnknownRoute {
            route,
            id: id.to_string(),
        })?;
    if device.kind != expected_kind {
        return Err(AudioAdapterError::WrongRouteKind {
            route,
            id: id.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(id: &str) -> AudioDevice {
        AudioDevice::new(id, format!("Output {id}"), AudioDeviceKind::Output)
            .expect("valid output fixture")
    }

    fn input(id: &str) -> AudioDevice {
        AudioDevice::new(id, format!("Input {id}"), AudioDeviceKind::Input)
            .expect("valid input fixture")
    }

    fn ready_state(generation: u64, volume: u8, muted: bool) -> AudioAuthoritativeState {
        AudioAuthoritativeState::new(
            generation,
            AudioServiceHealth::Ready,
            vec![output("sink.1"), output("sink.2"), input("source.1")],
            Some("sink.1".to_string()),
            Some("source.1".to_string()),
            volume,
            muted,
        )
        .expect("valid ready fixture")
    }

    #[test]
    fn unavailable_and_degraded_states_fail_closed_and_preserve_preferences() {
        let mut adapter = AudioServiceAdapter::with_preferences(65, false).unwrap();
        assert!(!adapter.controls_enabled());
        assert!(!adapter.backend_applied());
        assert!(adapter.next_reconciliation_request().unwrap().is_none());

        adapter.reconcile(ready_state(1, 65, false)).unwrap();
        assert!(adapter.controls_enabled());
        assert!(adapter.backend_applied());
        adapter.set_desired_volume(80).unwrap();
        assert_eq!(adapter.desired_volume_percent(), 80);
        assert!(!adapter.backend_applied());
        assert!(adapter.next_reconciliation_request().unwrap().is_some());

        let outcome = adapter
            .reconcile(
                AudioAuthoritativeState::unavailable(2, AudioServiceHealth::Degraded).unwrap(),
            )
            .unwrap();
        assert!(outcome.request_rejected_or_lost);
        assert!(!adapter.controls_enabled());
        assert!(!adapter.backend_applied());
        assert_eq!(adapter.desired_volume_percent(), 80);
        assert!(adapter.pending_request().is_none());
    }

    #[test]
    fn volume_and_mute_require_authoritative_reconciliation() {
        let mut adapter = AudioServiceAdapter::with_preferences(70, false).unwrap();
        adapter.reconcile(ready_state(1, 55, false)).unwrap();
        let volume = adapter
            .next_reconciliation_request()
            .unwrap()
            .expect("volume request");
        assert_eq!(volume.expected_generation(), 1);
        assert_eq!(volume.intent(), &AudioIntent::SetOutputVolume(70));
        assert!(!adapter.backend_applied());

        let outcome = adapter.reconcile(ready_state(2, 70, false)).unwrap();
        assert!(outcome.request_confirmed);
        assert!(adapter.backend_applied());

        assert!(adapter.set_desired_muted(true));
        let mute = adapter
            .next_reconciliation_request()
            .unwrap()
            .expect("mute request");
        assert_eq!(mute.intent(), &AudioIntent::SetOutputMuted(true));
        assert!(!adapter.backend_applied());
        assert!(
            adapter
                .reconcile(ready_state(3, 70, true))
                .unwrap()
                .request_confirmed
        );
        assert!(adapter.backend_applied());
    }

    #[test]
    fn devices_and_routes_are_bounded_typed_and_authoritative() {
        assert!(matches!(
            AudioDevice::new("bad/id", "Bad", AudioDeviceKind::Output),
            Err(AudioAdapterError::InvalidDeviceId)
        ));
        assert!(matches!(
            AudioAuthoritativeState::new(
                1,
                AudioServiceHealth::Ready,
                vec![output("sink.1")],
                Some("missing".to_string()),
                None,
                70,
                false,
            ),
            Err(AudioAdapterError::UnknownRoute { .. })
        ));
        assert!(matches!(
            AudioAuthoritativeState::new(
                1,
                AudioServiceHealth::Ready,
                vec![input("source.1")],
                Some("source.1".to_string()),
                None,
                70,
                false,
            ),
            Err(AudioAdapterError::WrongRouteKind { .. })
        ));

        let mut adapter = AudioServiceAdapter::with_preferences(70, false).unwrap();
        adapter.reconcile(ready_state(1, 70, false)).unwrap();
        adapter.set_desired_output("sink.2").unwrap();
        let route = adapter
            .next_reconciliation_request()
            .unwrap()
            .expect("route request");
        assert_eq!(
            route.intent(),
            &AudioIntent::SetDefaultOutput("sink.2".to_string())
        );
        let routed = AudioAuthoritativeState::new(
            2,
            AudioServiceHealth::Ready,
            vec![output("sink.1"), output("sink.2"), input("source.1")],
            Some("sink.2".to_string()),
            Some("source.1".to_string()),
            70,
            false,
        )
        .unwrap();
        assert!(adapter.reconcile(routed).unwrap().request_confirmed);
        assert_eq!(adapter.state().default_output(), Some("sink.2"));
        assert!(adapter.backend_applied());
    }

    #[test]
    fn stale_and_conflicting_generations_are_rejected_atomically() {
        let mut adapter = AudioServiceAdapter::default();
        let first = ready_state(4, 70, false);
        adapter.reconcile(first.clone()).unwrap();
        assert!(!adapter.reconcile(first).unwrap().generation_advanced);
        assert!(matches!(
            adapter.reconcile(ready_state(3, 70, false)),
            Err(AudioAdapterError::StaleGeneration { .. })
        ));
        assert!(matches!(
            adapter.reconcile(ready_state(4, 80, false)),
            Err(AudioAdapterError::ConflictingGeneration(4))
        ));
        assert_eq!(adapter.state().output_volume_percent(), 70);
    }

    #[test]
    fn service_snapshot_does_not_become_ready_without_an_output_route() {
        let state = AudioAuthoritativeState::new(
            1,
            AudioServiceHealth::Ready,
            vec![input("source.1")],
            None,
            Some("source.1".to_string()),
            70,
            false,
        )
        .unwrap();
        assert!(!state.controls_enabled());
        let mut adapter = AudioServiceAdapter::default();
        adapter.reconcile(state).unwrap();
        assert!(!adapter.controls_enabled());
        assert!(!adapter.backend_applied());
    }
}
