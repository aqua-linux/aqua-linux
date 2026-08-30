#[cfg(target_os = "linux")]
use aqua_service_adapters::{
    AudioBackend, AudioBackendDriveError, AudioDeviceKind, AudioServiceAdapter, PipeWireApi,
    PipeWireApiPhase, PipeWireApiSnapshot, PipeWireApiTransport, PipeWireTransportError,
    WirePlumberNativeApi, WirePlumberNativeError, MAX_AUDIO_CONTROL_SUBMISSION_ATTEMPTS,
};
#[cfg(target_os = "linux")]
use std::fmt;
#[cfg(target_os = "linux")]
use std::io::{self, Write};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

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
fn run_submission_budget() -> Result<(), Box<dyn std::error::Error>> {
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
fn wait_for_native_snapshot(
    native: &mut WirePlumberNativeApi,
    description: &str,
    predicate: impl Fn(&PipeWireApiSnapshot) -> bool,
) -> Result<PipeWireApiSnapshot, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let snapshot = native.synchronized_snapshot()?;
        if predicate(&snapshot) {
            return Ok(snapshot);
        }
        if Instant::now() >= deadline {
            return Err(format!("timed out waiting for {description}").into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(target_os = "linux")]
fn run_route_generation_loss() -> Result<(), Box<dyn std::error::Error>> {
    const SELECTED_VOLUME: u8 = 63;
    const FALLBACK_VOLUME: u8 = 37;

    let mut native = WirePlumberNativeApi::connect(Duration::from_secs(5))?;
    let initial = wait_for_native_snapshot(&mut native, "two authoritative outputs", |snapshot| {
        snapshot.phase() == PipeWireApiPhase::Ready
            && snapshot
                .nodes()
                .iter()
                .filter(|node| node.kind() == AudioDeviceKind::Output)
                .count()
                == 2
    })?;
    let selected = initial
        .nodes()
        .iter()
        .find(|node| node.kind() == AudioDeviceKind::Output && node.name().contains("05.0"))
        .ok_or("PCI 05.0 output node is required")?
        .name()
        .to_string();
    let fallback = initial
        .nodes()
        .iter()
        .find(|node| node.kind() == AudioDeviceKind::Output && node.name().contains("04.0"))
        .ok_or("PCI 04.0 output node is required")?
        .name()
        .to_string();

    native.set_configured_default_output(&selected)?;
    wait_for_native_snapshot(&mut native, "selected PCI 05.0 output", |snapshot| {
        snapshot.default_output() == Some(selected.as_str())
    })?;
    native.set_output_volume(&selected, SELECTED_VOLUME)?;
    wait_for_native_snapshot(&mut native, "selected output volume", |snapshot| {
        snapshot.default_output() == Some(selected.as_str())
            && snapshot
                .nodes()
                .iter()
                .find(|node| node.name() == selected)
                .is_some_and(|node| node.volume_percent() == SELECTED_VOLUME)
    })?;

    native.set_configured_default_output(&fallback)?;
    wait_for_native_snapshot(&mut native, "fallback PCI 04.0 output", |snapshot| {
        snapshot.default_output() == Some(fallback.as_str())
    })?;
    native.set_output_volume(&fallback, FALLBACK_VOLUME)?;
    wait_for_native_snapshot(&mut native, "prepared fallback volume", |snapshot| {
        snapshot.default_output() == Some(fallback.as_str())
            && snapshot
                .nodes()
                .iter()
                .find(|node| node.name() == fallback)
                .is_some_and(|node| node.volume_percent() == FALLBACK_VOLUME)
    })?;

    native.set_configured_default_output(&selected)?;
    let selected_state =
        wait_for_native_snapshot(&mut native, "selected output restoration", |snapshot| {
            snapshot.default_output() == Some(selected.as_str())
                && snapshot
                    .nodes()
                    .iter()
                    .find(|node| node.name() == selected)
                    .is_some_and(|node| node.volume_percent() == SELECTED_VOLUME)
        })?;
    let selected_muted = selected_state
        .nodes()
        .iter()
        .find(|node| node.name() == selected)
        .ok_or("selected output disappeared before control submission")?
        .muted();

    let mut backend = PipeWireApiTransport::new(native);
    let mut adapter = AudioServiceAdapter::with_preferences(FALLBACK_VOLUME, selected_muted)?;
    let submitted = adapter.drive_backend_once(&mut backend)?;
    let pending = adapter
        .pending_request()
        .ok_or("volume request did not remain pending")?;
    if submitted.submitted_request_id != Some(pending.id())
        || pending.target_output() != Some(selected.as_str())
        || adapter.backend_applied()
    {
        return Err("pending control was not bound to the selected output".into());
    }

    println!(
        "[AQUA-AUDIO] stage=adapter-route-generation status=pending target_bound=true selected_route=true fallback_prepared=true fallback_matches_desired=true"
    );
    io::stdout().flush()?;

    let deadline = Instant::now() + Duration::from_secs(30);
    let fallback_state = loop {
        let state = backend.authoritative_state()?;
        if state.default_output() == Some(fallback.as_str())
            && state.devices().iter().all(|device| device.id() != selected)
            && state.output_volume_percent() == FALLBACK_VOLUME
        {
            break state;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for authoritative fallback after route loss".into());
        }
        thread::sleep(Duration::from_millis(100));
    };
    let outcome = adapter.reconcile(fallback_state)?;
    if outcome.request_confirmed
        || !outcome.request_rejected_or_lost
        || adapter.pending_request().is_some()
        || adapter.next_reconciliation_request()?.is_some()
        || !adapter.backend_applied()
    {
        return Err("lost selected-route control produced a false acknowledgement".into());
    }

    println!(
        "[AQUA-AUDIO] stage=adapter-route-generation status=ok target_bound=true route_changed=true old_request_confirmed=false request_rejected_or_lost=true fallback_matches_desired=true no_resubmission=true"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_mute_route_generation_loss() -> Result<(), Box<dyn std::error::Error>> {
    const ROUTE_VOLUME: u8 = 42;

    let mut native = WirePlumberNativeApi::connect(Duration::from_secs(5))?;
    let initial = wait_for_native_snapshot(&mut native, "two authoritative outputs", |snapshot| {
        snapshot.phase() == PipeWireApiPhase::Ready
            && snapshot
                .nodes()
                .iter()
                .filter(|node| node.kind() == AudioDeviceKind::Output)
                .count()
                == 2
    })?;
    let selected = initial
        .nodes()
        .iter()
        .find(|node| node.kind() == AudioDeviceKind::Output && node.name().contains("05.0"))
        .ok_or("PCI 05.0 output node is required")?
        .name()
        .to_string();
    let fallback = initial
        .nodes()
        .iter()
        .find(|node| node.kind() == AudioDeviceKind::Output && node.name().contains("04.0"))
        .ok_or("PCI 04.0 output node is required")?
        .name()
        .to_string();

    native.set_configured_default_output(&selected)?;
    native.set_output_volume(&selected, ROUTE_VOLUME)?;
    native.set_output_muted(&selected, false)?;
    wait_for_native_snapshot(
        &mut native,
        "unmuted selected PCI 05.0 output",
        |snapshot| {
            snapshot.default_output() == Some(selected.as_str())
                && snapshot
                    .nodes()
                    .iter()
                    .find(|node| node.name() == selected)
                    .is_some_and(|node| node.volume_percent() == ROUTE_VOLUME && !node.muted())
        },
    )?;

    native.set_configured_default_output(&fallback)?;
    native.set_output_volume(&fallback, ROUTE_VOLUME)?;
    native.set_output_muted(&fallback, true)?;
    wait_for_native_snapshot(&mut native, "muted fallback PCI 04.0 output", |snapshot| {
        snapshot.default_output() == Some(fallback.as_str())
            && snapshot
                .nodes()
                .iter()
                .find(|node| node.name() == fallback)
                .is_some_and(|node| node.volume_percent() == ROUTE_VOLUME && node.muted())
    })?;

    native.set_configured_default_output(&selected)?;
    wait_for_native_snapshot(&mut native, "selected output restoration", |snapshot| {
        snapshot.default_output() == Some(selected.as_str())
            && snapshot
                .nodes()
                .iter()
                .find(|node| node.name() == selected)
                .is_some_and(|node| node.volume_percent() == ROUTE_VOLUME && !node.muted())
    })?;

    let mut backend = PipeWireApiTransport::new(native);
    let mut adapter = AudioServiceAdapter::with_preferences(ROUTE_VOLUME, true)?;
    let submitted = adapter.drive_backend_once(&mut backend)?;
    let pending = adapter
        .pending_request()
        .ok_or("mute request did not remain pending")?;
    if submitted.submitted_request_id != Some(pending.id())
        || pending.target_output() != Some(selected.as_str())
        || adapter.backend_applied()
    {
        return Err("pending mute was not bound to the selected output".into());
    }

    println!(
        "[AQUA-AUDIO] stage=adapter-mute-route-generation status=pending target_bound=true selected_route=true fallback_prepared=true fallback_matches_desired=true"
    );
    io::stdout().flush()?;

    let deadline = Instant::now() + Duration::from_secs(30);
    let fallback_state = loop {
        let state = backend.authoritative_state()?;
        if state.default_output() == Some(fallback.as_str())
            && state.devices().iter().all(|device| device.id() != selected)
            && state.output_volume_percent() == ROUTE_VOLUME
            && state.output_muted()
        {
            break state;
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for muted fallback after route loss".into());
        }
        thread::sleep(Duration::from_millis(100));
    };
    let outcome = adapter.reconcile(fallback_state)?;
    if outcome.request_confirmed
        || !outcome.request_rejected_or_lost
        || adapter.pending_request().is_some()
        || adapter.next_reconciliation_request()?.is_some()
        || !adapter.backend_applied()
    {
        return Err("lost selected-route mute produced a false acknowledgement".into());
    }

    println!(
        "[AQUA-AUDIO] stage=adapter-mute-route-generation status=ok target_bound=true route_changed=true old_request_confirmed=false request_rejected_or_lost=true fallback_matches_desired=true no_resubmission=true"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn main() {
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "submission-budget".to_string());
    let (stage, result) = match mode.as_str() {
        "submission-budget" => ("adapter-submission-budget", run_submission_budget()),
        "route-generation-loss" => ("adapter-route-generation", run_route_generation_loss()),
        "mute-route-generation-loss" => (
            "adapter-mute-route-generation",
            run_mute_route_generation_loss(),
        ),
        _ => {
            eprintln!("unsupported aqua audio adapter probe mode: {mode}");
            std::process::exit(2);
        }
    };
    if let Err(error) = result {
        eprintln!("aqua-audio-adapter-probe: {error}");
        eprintln!("[AQUA-AUDIO] stage={stage} status=failed detail={error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("aqua-audio-adapter-probe requires Linux");
    std::process::exit(1);
}
