use crate::{
    AudioAdapterError, AudioAuthoritativeState, AudioBackend, AudioDevice, AudioDeviceKind,
    AudioIntent, AudioRequest, AudioServiceHealth, MAX_AUDIO_DEVICES,
};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeWireApiPhase {
    Disconnected,
    Connecting,
    Synchronizing,
    Ready,
    Degraded,
}

impl PipeWireApiPhase {
    const fn health(self) -> AudioServiceHealth {
        match self {
            Self::Disconnected => AudioServiceHealth::Unavailable,
            Self::Connecting | Self::Synchronizing => AudioServiceHealth::Starting,
            Self::Ready => AudioServiceHealth::Ready,
            Self::Degraded => AudioServiceHealth::Degraded,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeWireApiNode {
    name: String,
    description: String,
    kind: AudioDeviceKind,
    volume_percent: u8,
    muted: bool,
}

impl PipeWireApiNode {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        kind: AudioDeviceKind,
        volume_percent: u8,
        muted: bool,
    ) -> Result<Self, PipeWireTransportError<std::convert::Infallible>> {
        let node = Self {
            name: name.into(),
            description: description.into(),
            kind,
            volume_percent,
            muted,
        };
        node.validate()?;
        Ok(node)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub const fn kind(&self) -> AudioDeviceKind {
        self.kind
    }

    pub const fn volume_percent(&self) -> u8 {
        self.volume_percent
    }

    pub const fn muted(&self) -> bool {
        self.muted
    }

    fn validate<E>(&self) -> Result<(), PipeWireTransportError<E>> {
        AudioDevice::new(self.name.clone(), self.description.clone(), self.kind)
            .map_err(PipeWireTransportError::Adapter)?;
        if self.volume_percent > 100 {
            return Err(PipeWireTransportError::Adapter(
                AudioAdapterError::InvalidVolume(self.volume_percent),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeWireApiSnapshot {
    generation: u64,
    phase: PipeWireApiPhase,
    nodes: Vec<PipeWireApiNode>,
    default_output: Option<String>,
    default_input: Option<String>,
}

impl PipeWireApiSnapshot {
    pub fn new(
        generation: u64,
        phase: PipeWireApiPhase,
        nodes: Vec<PipeWireApiNode>,
        default_output: Option<String>,
        default_input: Option<String>,
    ) -> Result<Self, PipeWireTransportError<std::convert::Infallible>> {
        let snapshot = Self {
            generation,
            phase,
            nodes,
            default_output,
            default_input,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn phase(&self) -> PipeWireApiPhase {
        self.phase
    }

    pub fn nodes(&self) -> &[PipeWireApiNode] {
        &self.nodes
    }

    pub fn default_output(&self) -> Option<&str> {
        self.default_output.as_deref()
    }

    pub fn default_input(&self) -> Option<&str> {
        self.default_input.as_deref()
    }

    fn validate<E>(&self) -> Result<(), PipeWireTransportError<E>> {
        if self.nodes.len() > MAX_AUDIO_DEVICES {
            return Err(PipeWireTransportError::Adapter(
                AudioAdapterError::TooManyDevices(self.nodes.len()),
            ));
        }
        let mut names = HashSet::with_capacity(self.nodes.len());
        for node in &self.nodes {
            node.validate()?;
            if !names.insert(node.name.as_str()) {
                return Err(PipeWireTransportError::DuplicateNodeName(node.name.clone()));
            }
        }
        self.validate_default(
            "output",
            self.default_output.as_deref(),
            AudioDeviceKind::Output,
        )?;
        self.validate_default(
            "input",
            self.default_input.as_deref(),
            AudioDeviceKind::Input,
        )?;
        if self.phase != PipeWireApiPhase::Ready
            && (self.default_output.is_some() || self.default_input.is_some())
        {
            return Err(PipeWireTransportError::DefaultsBeforeGraphSync);
        }
        Ok(())
    }

    fn validate_default<E>(
        &self,
        route: &'static str,
        name: Option<&str>,
        expected_kind: AudioDeviceKind,
    ) -> Result<(), PipeWireTransportError<E>> {
        let Some(name) = name else {
            return Ok(());
        };
        let Some(node) = self.nodes.iter().find(|node| node.name == name) else {
            return Err(PipeWireTransportError::UnknownNode {
                route,
                name: name.to_string(),
            });
        };
        if node.kind != expected_kind {
            return Err(PipeWireTransportError::WrongNodeKind {
                route,
                name: name.to_string(),
            });
        }
        Ok(())
    }
}

pub trait PipeWireApi {
    type Error;

    fn synchronized_snapshot(&mut self) -> Result<PipeWireApiSnapshot, Self::Error>;
    fn set_output_volume(&mut self, node_name: &str, volume_percent: u8)
        -> Result<(), Self::Error>;
    fn set_output_muted(&mut self, node_name: &str, muted: bool) -> Result<(), Self::Error>;
    fn set_configured_default_output(&mut self, node_name: &str) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct PipeWireApiTransport<A> {
    api: A,
    last_state: Option<AudioAuthoritativeState>,
}

impl<A> PipeWireApiTransport<A> {
    pub const fn new(api: A) -> Self {
        Self {
            api,
            last_state: None,
        }
    }

    pub fn api(&self) -> &A {
        &self.api
    }

    pub fn api_mut(&mut self) -> &mut A {
        &mut self.api
    }

    pub fn into_api(self) -> A {
        self.api
    }
}

impl<A: PipeWireApi> AudioBackend for PipeWireApiTransport<A> {
    type Error = PipeWireTransportError<A::Error>;

    fn authoritative_state(&mut self) -> Result<AudioAuthoritativeState, Self::Error> {
        let snapshot = self
            .api
            .synchronized_snapshot()
            .map_err(PipeWireTransportError::Api)?;
        snapshot.validate()?;
        let state = map_snapshot(snapshot)?;
        self.last_state = Some(state.clone());
        Ok(state)
    }

    fn submit(&mut self, request: &AudioRequest) -> Result<(), Self::Error> {
        let state = self
            .last_state
            .as_ref()
            .ok_or(PipeWireTransportError::SnapshotRequired)?;
        if request.expected_generation() != state.generation() {
            return Err(PipeWireTransportError::GenerationMismatch {
                expected: request.expected_generation(),
                current: state.generation(),
            });
        }
        if !state.controls_enabled() {
            return Err(PipeWireTransportError::ServiceNotReady);
        }
        match request.intent() {
            AudioIntent::SetOutputVolume(value) => {
                let output = state
                    .default_output()
                    .ok_or(PipeWireTransportError::DefaultOutputRequired)?;
                self.api
                    .set_output_volume(output, *value)
                    .map_err(PipeWireTransportError::Api)
            }
            AudioIntent::SetOutputMuted(value) => {
                let output = state
                    .default_output()
                    .ok_or(PipeWireTransportError::DefaultOutputRequired)?;
                self.api
                    .set_output_muted(output, *value)
                    .map_err(PipeWireTransportError::Api)
            }
            AudioIntent::SetDefaultOutput(name) => {
                let node = state
                    .devices()
                    .iter()
                    .find(|node| node.id() == name)
                    .ok_or_else(|| PipeWireTransportError::UnknownNode {
                        route: "output",
                        name: name.clone(),
                    })?;
                if node.kind() != AudioDeviceKind::Output {
                    return Err(PipeWireTransportError::WrongNodeKind {
                        route: "output",
                        name: name.clone(),
                    });
                }
                self.api
                    .set_configured_default_output(name)
                    .map_err(PipeWireTransportError::Api)
            }
        }
    }
}

fn map_snapshot<E>(
    snapshot: PipeWireApiSnapshot,
) -> Result<AudioAuthoritativeState, PipeWireTransportError<E>> {
    let health = snapshot.phase.health();
    if health != AudioServiceHealth::Ready {
        return AudioAuthoritativeState::unavailable(snapshot.generation, health)
            .map_err(PipeWireTransportError::Adapter);
    }
    let output_state = snapshot
        .default_output
        .as_deref()
        .and_then(|name| snapshot.nodes.iter().find(|node| node.name == name));
    let output_volume = output_state.map_or(0, PipeWireApiNode::volume_percent);
    let output_muted = output_state.is_none_or(PipeWireApiNode::muted);
    let devices = snapshot
        .nodes
        .into_iter()
        .map(|node| AudioDevice::new(node.name, node.description, node.kind))
        .collect::<Result<Vec<_>, _>>()
        .map_err(PipeWireTransportError::Adapter)?;
    AudioAuthoritativeState::new(
        snapshot.generation,
        health,
        devices,
        snapshot.default_output,
        snapshot.default_input,
        output_volume,
        output_muted,
    )
    .map_err(PipeWireTransportError::Adapter)
}

#[derive(Debug, PartialEq, Eq)]
pub enum PipeWireTransportError<E> {
    Api(E),
    Adapter(AudioAdapterError),
    DuplicateNodeName(String),
    UnknownNode { route: &'static str, name: String },
    WrongNodeKind { route: &'static str, name: String },
    DefaultsBeforeGraphSync,
    SnapshotRequired,
    GenerationMismatch { expected: u64, current: u64 },
    ServiceNotReady,
    DefaultOutputRequired,
}

impl<E: fmt::Display> fmt::Display for PipeWireTransportError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(error) => write!(formatter, "PipeWire/WirePlumber API error: {error}"),
            Self::Adapter(error) => write!(formatter, "audio adapter error: {error}"),
            Self::DuplicateNodeName(name) => write!(formatter, "duplicate node name: {name}"),
            Self::UnknownNode { route, name } => {
                write!(formatter, "unknown {route} node: {name}")
            }
            Self::WrongNodeKind { route, name } => {
                write!(formatter, "wrong {route} node kind: {name}")
            }
            Self::DefaultsBeforeGraphSync => {
                formatter.write_str("defaults received before graph synchronization")
            }
            Self::SnapshotRequired => formatter.write_str("synchronized snapshot required"),
            Self::GenerationMismatch { expected, current } => write!(
                formatter,
                "snapshot generation mismatch: expected {expected}, current {current}"
            ),
            Self::ServiceNotReady => formatter.write_str("audio service is not ready"),
            Self::DefaultOutputRequired => formatter.write_str("default output required"),
        }
    }
}

impl<E> std::error::Error for PipeWireTransportError<E> where
    E: fmt::Debug + std::error::Error + 'static
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AudioBackendDriveError, AudioServiceAdapter};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum FakeError {
        Rejected,
    }

    impl fmt::Display for FakeError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{self:?}")
        }
    }

    impl std::error::Error for FakeError {}

    #[derive(Debug)]
    struct FakeApi {
        snapshot: PipeWireApiSnapshot,
        calls: Vec<String>,
        reject_next: bool,
    }

    impl FakeApi {
        fn ready(generation: u64, volume: u8, muted: bool) -> Self {
            Self {
                snapshot: ready_snapshot(generation, volume, muted),
                calls: Vec::new(),
                reject_next: false,
            }
        }

        fn record(&mut self, call: String) -> Result<(), FakeError> {
            if self.reject_next {
                self.reject_next = false;
                return Err(FakeError::Rejected);
            }
            self.calls.push(call);
            Ok(())
        }
    }

    impl PipeWireApi for FakeApi {
        type Error = FakeError;

        fn synchronized_snapshot(&mut self) -> Result<PipeWireApiSnapshot, Self::Error> {
            Ok(self.snapshot.clone())
        }

        fn set_output_volume(
            &mut self,
            node_name: &str,
            volume_percent: u8,
        ) -> Result<(), Self::Error> {
            self.record(format!("volume:{node_name}:{volume_percent}"))
        }

        fn set_output_muted(&mut self, node_name: &str, muted: bool) -> Result<(), Self::Error> {
            self.record(format!("mute:{node_name}:{muted}"))
        }

        fn set_configured_default_output(&mut self, node_name: &str) -> Result<(), Self::Error> {
            self.record(format!("default:{node_name}"))
        }
    }

    fn node(name: &str, kind: AudioDeviceKind, volume: u8, muted: bool) -> PipeWireApiNode {
        PipeWireApiNode::new(name, format!("Node {name}"), kind, volume, muted).unwrap()
    }

    fn ready_snapshot(generation: u64, volume: u8, muted: bool) -> PipeWireApiSnapshot {
        PipeWireApiSnapshot::new(
            generation,
            PipeWireApiPhase::Ready,
            vec![
                node("alsa_output.pci", AudioDeviceKind::Output, volume, muted),
                node("usb_output.dac", AudioDeviceKind::Output, 45, false),
                node("alsa_input.pci", AudioDeviceKind::Input, 80, false),
            ],
            Some("alsa_output.pci".to_string()),
            Some("alsa_input.pci".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn phases_fail_closed_until_the_native_graph_is_synchronized() {
        for (phase, health) in [
            (
                PipeWireApiPhase::Disconnected,
                AudioServiceHealth::Unavailable,
            ),
            (PipeWireApiPhase::Connecting, AudioServiceHealth::Starting),
            (
                PipeWireApiPhase::Synchronizing,
                AudioServiceHealth::Starting,
            ),
            (PipeWireApiPhase::Degraded, AudioServiceHealth::Degraded),
        ] {
            let snapshot = PipeWireApiSnapshot::new(1, phase, Vec::new(), None, None).unwrap();
            let mut transport = PipeWireApiTransport::new(FakeApi {
                snapshot,
                calls: Vec::new(),
                reject_next: false,
            });
            let state = transport.authoritative_state().unwrap();
            assert_eq!(state.health(), health);
            assert!(!state.controls_enabled());
        }
        assert!(matches!(
            PipeWireApiSnapshot::new(
                1,
                PipeWireApiPhase::Synchronizing,
                vec![node("sink.one", AudioDeviceKind::Output, 70, false)],
                Some("sink.one".to_string()),
                None,
            ),
            Err(PipeWireTransportError::DefaultsBeforeGraphSync)
        ));
    }

    #[test]
    fn synchronized_snapshot_maps_nodes_defaults_and_default_output_props() {
        let mut transport = PipeWireApiTransport::new(FakeApi::ready(7, 63, true));
        let state = transport.authoritative_state().unwrap();
        assert_eq!(state.generation(), 7);
        assert_eq!(state.devices().len(), 3);
        assert_eq!(state.default_output(), Some("alsa_output.pci"));
        assert_eq!(state.default_input(), Some("alsa_input.pci"));
        assert_eq!(state.output_volume_percent(), 63);
        assert!(state.output_muted());
        assert!(state.controls_enabled());
    }

    #[test]
    fn adapter_drive_submits_native_calls_and_waits_for_acknowledgement() {
        let mut adapter = AudioServiceAdapter::with_preferences(75, false).unwrap();
        let mut transport = PipeWireApiTransport::new(FakeApi::ready(1, 50, false));
        let first = adapter.drive_backend_once(&mut transport).unwrap();
        assert_eq!(first.submitted_request_id, Some(1));
        assert_eq!(transport.api().calls, ["volume:alsa_output.pci:75"]);
        assert!(!adapter.backend_applied());

        transport.api_mut().snapshot = ready_snapshot(2, 75, false);
        let second = adapter.drive_backend_once(&mut transport).unwrap();
        assert!(second.reconcile.request_confirmed);
        assert_eq!(second.submitted_request_id, None);
        assert!(adapter.backend_applied());
    }

    #[test]
    fn route_changes_use_configured_default_api_and_require_new_snapshot() {
        let mut adapter = AudioServiceAdapter::with_preferences(70, false).unwrap();
        let mut transport = PipeWireApiTransport::new(FakeApi::ready(1, 70, false));
        adapter.drive_backend_once(&mut transport).unwrap();
        adapter.set_desired_output("usb_output.dac").unwrap();
        let sent = adapter.drive_backend_once(&mut transport).unwrap();
        assert_eq!(sent.submitted_request_id, Some(1));
        assert_eq!(transport.api().calls, ["default:usb_output.dac"]);
        assert_eq!(adapter.state().default_output(), Some("alsa_output.pci"));
        assert!(!adapter.backend_applied());

        transport.api_mut().snapshot = PipeWireApiSnapshot::new(
            2,
            PipeWireApiPhase::Ready,
            vec![
                node("alsa_output.pci", AudioDeviceKind::Output, 70, false),
                node("usb_output.dac", AudioDeviceKind::Output, 70, false),
            ],
            Some("usb_output.dac".to_string()),
            None,
        )
        .unwrap();
        assert!(
            adapter
                .drive_backend_once(&mut transport)
                .unwrap()
                .reconcile
                .request_confirmed
        );
        assert!(adapter.backend_applied());
    }

    #[test]
    fn failed_native_submission_is_retryable_and_never_applied() {
        let mut adapter = AudioServiceAdapter::with_preferences(80, false).unwrap();
        let mut api = FakeApi::ready(1, 40, false);
        api.reject_next = true;
        let mut transport = PipeWireApiTransport::new(api);
        assert!(matches!(
            adapter.drive_backend_once(&mut transport),
            Err(AudioBackendDriveError::Backend(
                PipeWireTransportError::Api(FakeError::Rejected)
            ))
        ));
        assert!(adapter.pending_request().is_none());
        assert!(!adapter.backend_applied());
        assert_eq!(
            adapter
                .drive_backend_once(&mut transport)
                .unwrap()
                .submitted_request_id,
            Some(2)
        );
        assert_eq!(transport.api().calls, ["volume:alsa_output.pci:80"]);
    }

    #[test]
    fn repeated_submission_failure_is_bounded_until_a_new_snapshot_generation() {
        let mut adapter = AudioServiceAdapter::with_preferences(80, false).unwrap();
        let mut transport = PipeWireApiTransport::new(FakeApi::ready(1, 40, false));

        for expected_failures in 1..=crate::MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS {
            transport.api_mut().reject_next = true;
            assert!(matches!(
                adapter.drive_backend_once(&mut transport),
                Err(AudioBackendDriveError::Backend(
                    PipeWireTransportError::Api(FakeError::Rejected)
                ))
            ));
            assert_eq!(adapter.consecutive_submission_failures(), expected_failures);
            assert_eq!(
                adapter.submission_retry_exhausted(),
                expected_failures == crate::MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS
            );
            assert!(!adapter.backend_applied());
        }

        let blocked = adapter.drive_backend_once(&mut transport).unwrap();
        assert_eq!(blocked.submitted_request_id, None);
        assert!(transport.api().calls.is_empty());
        assert_eq!(adapter.desired_volume_percent(), 80);

        transport.api_mut().snapshot = ready_snapshot(2, 40, false);
        let resumed = adapter.drive_backend_once(&mut transport).unwrap();
        assert_eq!(resumed.submitted_request_id, Some(4));
        assert!(!adapter.submission_retry_exhausted());
        assert_eq!(adapter.consecutive_submission_failures(), 0);
        assert_eq!(transport.api().calls, ["volume:alsa_output.pci:80"]);
        assert!(!adapter.backend_applied());
    }
}
