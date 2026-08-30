#[cfg(target_os = "linux")]
use aqua_service_adapters::{
    AudioBackendDriveError, AudioServiceAdapter, PipeWireApi, PipeWireApiSnapshot,
    PipeWireApiTransport, PipeWireTransportError, WirePlumberNativeApi, WirePlumberNativeError,
    MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS,
};
#[cfg(target_os = "linux")]
use std::fmt;
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
#[derive(Debug)]
enum ProbeBackendError {
    InjectedSubmission,
    Native(WirePlumberNativeError),
}

#[cfg(target_os = "linux")]
impl fmt::Display for ProbeBackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InjectedSubmission => formatter.write_str("injected control submission failure"),
            Self::Native(error) => write!(formatter, "{error}"),
        }
    }
}

#[cfg(target_os = "linux")]
impl std::error::Error for ProbeBackendError {}

#[cfg(target_os = "linux")]
struct SubmissionFaultApi {
    native: WirePlumberNativeApi,
    remaining_failures: u8,
    submission_calls: u8,
}

#[cfg(target_os = "linux")]
impl SubmissionFaultApi {
    fn new(native: WirePlumberNativeApi) -> Self {
        Self {
            native,
            remaining_failures: MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS,
            submission_calls: 0,
        }
    }

    fn reject_or_apply(
        &mut self,
        apply: impl FnOnce(&mut WirePlumberNativeApi) -> Result<(), WirePlumberNativeError>,
    ) -> Result<(), ProbeBackendError> {
        self.submission_calls = self.submission_calls.saturating_add(1);
        if self.remaining_failures > 0 {
            self.remaining_failures -= 1;
            return Err(ProbeBackendError::InjectedSubmission);
        }
        apply(&mut self.native).map_err(ProbeBackendError::Native)
    }
}

#[cfg(target_os = "linux")]
impl PipeWireApi for SubmissionFaultApi {
    type Error = ProbeBackendError;

    fn synchronized_snapshot(&mut self) -> Result<PipeWireApiSnapshot, Self::Error> {
        self.native
            .synchronized_snapshot()
            .map_err(ProbeBackendError::Native)
    }

    fn set_output_volume(
        &mut self,
        node_name: &str,
        volume_percent: u8,
    ) -> Result<(), Self::Error> {
        self.reject_or_apply(|native| native.set_output_volume(node_name, volume_percent))
    }

    fn set_output_muted(&mut self, node_name: &str, muted: bool) -> Result<(), Self::Error> {
        self.reject_or_apply(|native| native.set_output_muted(node_name, muted))
    }

    fn set_configured_default_output(&mut self, node_name: &str) -> Result<(), Self::Error> {
        self.reject_or_apply(|native| native.set_configured_default_output(node_name))
    }
}

#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let native = WirePlumberNativeApi::connect(Duration::from_secs(5))?;
    let mut fault_api = SubmissionFaultApi::new(native);
    let initial = fault_api.synchronized_snapshot()?;
    let output_name = initial
        .default_output()
        .ok_or("authoritative default output is required")?
        .to_string();
    let output = initial
        .nodes()
        .iter()
        .find(|node| node.name() == output_name)
        .ok_or("authoritative default output node is required")?;
    let initial_volume = output.volume_percent();
    let desired_volume = if initial_volume >= 50 {
        initial_volume - 17
    } else {
        initial_volume + 17
    };
    let kick_volume = if desired_volume == 100 {
        99
    } else {
        desired_volume + 1
    };
    let initial_muted = output.muted();
    let initial_generation = initial.generation();

    let mut backend = PipeWireApiTransport::new(fault_api);
    let mut adapter = AudioServiceAdapter::with_preferences(desired_volume, initial_muted)?;

    for expected_failures in 1..=MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS {
        match adapter.drive_backend_once(&mut backend) {
            Err(AudioBackendDriveError::Backend(PipeWireTransportError::Api(
                ProbeBackendError::InjectedSubmission,
            ))) => {}
            result => return Err(format!("unexpected failed-submission result: {result:?}").into()),
        }
        if adapter.state().generation() != initial_generation
            || adapter.consecutive_submission_failures() != expected_failures
        {
            return Err("submission budget advanced on an unchanged native graph".into());
        }
    }
    if !adapter.submission_retry_exhausted() {
        return Err("submission retry budget did not become exhausted".into());
    }

    let blocked = adapter.drive_backend_once(&mut backend)?;
    if blocked.submitted_request_id.is_some()
        || backend.api().submission_calls != MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS
    {
        return Err("an extra native submission escaped the generation budget".into());
    }

    backend
        .api_mut()
        .native
        .set_output_volume(&output_name, kick_volume)?;
    let recovery = adapter.drive_backend_once(&mut backend)?;
    if !recovery.reconcile.generation_advanced || recovery.submitted_request_id.is_none() {
        return Err("new authoritative generation did not reopen submission".into());
    }
    let acknowledgement = adapter.drive_backend_once(&mut backend)?;
    if !acknowledgement.reconcile.request_confirmed || !adapter.backend_applied() {
        return Err("recovered native control was not authoritatively acknowledged".into());
    }

    println!(
        "[AQUA-AUDIO] stage=adapter-submission-budget status=ok backend=aqua-audio-native generation_stable=true failed_submissions={} fourth_submission_blocked=true recovery_generation_advanced=true control_after=true authoritative_ack=true",
        MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn main() {
    if let Err(error) = run() {
        eprintln!("[AQUA-AUDIO] stage=adapter-submission-budget status=failed detail={error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("aqua-audio-adapter-probe requires Linux");
    std::process::exit(1);
}
