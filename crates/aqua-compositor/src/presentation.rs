pub const MAX_PRESENTATION_SAMPLES: usize = 8;
pub const MAX_PRESENTATION_EVENTS: u32 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationEvidenceTarget {
    HostFixture,
    QemuTcg,
    Physical,
}

impl PresentationEvidenceTarget {
    pub const fn id(self) -> &'static str {
        match self {
            Self::HostFixture => "host-fixture",
            Self::QemuTcg => "qemu-tcg",
            Self::Physical => "physical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationPath {
    ProductionGbmKms,
    DiagnosticReadback,
    LegacyCpuCopy,
}

impl PresentationPath {
    pub const fn id(self) -> &'static str {
        match self {
            Self::ProductionGbmKms => "production-gbm-kms",
            Self::DiagnosticReadback => "diagnostic-readback",
            Self::LegacyCpuCopy => "legacy-cpu-copy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationWorkload {
    Idle,
    WindowInteraction,
    Animation,
    MultiClient,
}

impl PresentationWorkload {
    pub const ALL: [Self; 4] = [
        Self::Idle,
        Self::WindowInteraction,
        Self::Animation,
        Self::MultiClient,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::WindowInteraction => "window-interaction",
            Self::Animation => "animation",
            Self::MultiClient => "multi-client",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationBudget {
    pub max_frame_time_us: u32,
    pub max_input_to_present_us: u32,
    pub max_dropped_frames: u32,
    pub max_cpu_time_us: u64,
    pub max_memory_growth_kib: u64,
}

impl PresentationBudget {
    pub const fn is_bounded(self) -> bool {
        self.max_frame_time_us > 0
            && self.max_input_to_present_us > 0
            && self.max_cpu_time_us > 0
            && self.max_memory_growth_kib > 0
    }
}

pub const QEMU_TCG_BOCHS_V1_BUDGET: PresentationBudget = PresentationBudget {
    max_frame_time_us: 50_000,
    max_input_to_present_us: 60_000_000,
    max_dropped_frames: 0,
    max_cpu_time_us: 180_000_000,
    max_memory_growth_kib: 163_840,
};

pub const QEMU_TCG_BOCHS_SOAK_V1_BUDGET: PresentationBudget = PresentationBudget {
    max_frame_time_us: 50_000,
    max_input_to_present_us: 60_000_000,
    max_dropped_frames: 0,
    max_cpu_time_us: 720_000_000,
    max_memory_growth_kib: 163_840,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationSample {
    pub target: PresentationEvidenceTarget,
    pub path: PresentationPath,
    pub workload: PresentationWorkload,
    pub observation_window_ms: u32,
    pub frames_requested: u32,
    pub frames_presented: u32,
    pub dropped_frames: u32,
    pub page_flip_events: u32,
    pub frame_callbacks_sent: u32,
    pub damage_commits: u32,
    pub full_frame_readbacks: u32,
    pub cpu_framebuffer_copies: u32,
    pub settled_idle_observations: u32,
    pub settled_idle_repaints: u32,
    pub repeating_repaint_timer_after_settle: bool,
    pub max_frame_time_us: Option<u32>,
    pub max_input_to_present_us: Option<u32>,
    pub cpu_time_us: Option<u64>,
    pub memory_growth_kib: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationEventSnapshot {
    pub target: PresentationEvidenceTarget,
    pub path: PresentationPath,
    pub workload: PresentationWorkload,
    pub frames_requested: u32,
    pub frames_presented: u32,
    pub page_flip_events: u32,
    pub frame_callbacks_sent: u32,
    pub damage_commits: u32,
    pub full_frame_readbacks: u32,
    pub input_to_present_samples: u32,
    pub settled_idle_observations: u32,
    pub settled_idle_repaints: u32,
    pub repeating_repaint_timer_after_settle: bool,
    pub cpu_framebuffer_copies: u32,
    pub max_frame_time_us: Option<u32>,
    pub max_input_to_present_us: Option<u32>,
    pub observation_window_ms: Option<u32>,
    pub cpu_time_us: Option<u64>,
    pub memory_growth_kib: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentationTelemetryError {
    EventLimitExceeded(&'static str),
    NoOutstandingFrame,
    InvalidFrameTime,
    InvalidInputToPresentTime,
    InvalidObservationWindow,
    IncompleteFrameAccounting,
}

impl std::fmt::Display for PresentationTelemetryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLimitExceeded(event) => {
                write!(
                    formatter,
                    "presentation telemetry event limit exceeded: {event}"
                )
            }
            Self::NoOutstandingFrame => formatter.write_str("no requested frame is outstanding"),
            Self::InvalidFrameTime => formatter.write_str("frame time must be non-zero"),
            Self::InvalidInputToPresentTime => {
                formatter.write_str("input-to-present time must be non-zero")
            }
            Self::InvalidObservationWindow => {
                formatter.write_str("observation window must be non-zero")
            }
            Self::IncompleteFrameAccounting => {
                formatter.write_str("requested frames are not fully presented or dropped")
            }
        }
    }
}

impl std::error::Error for PresentationTelemetryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationTelemetry {
    target: PresentationEvidenceTarget,
    path: PresentationPath,
    workload: PresentationWorkload,
    frames_requested: u32,
    frames_presented: u32,
    dropped_frames: u32,
    page_flip_events: u32,
    frame_callbacks_sent: u32,
    damage_commits: u32,
    full_frame_readbacks: u32,
    cpu_framebuffer_copies: u32,
    settled_idle_observations: u32,
    settled_idle_repaints: u32,
    repeating_repaint_timer_after_settle: bool,
    max_frame_time_us: Option<u32>,
    max_input_to_present_us: Option<u32>,
    input_to_present_samples: u32,
    observation_window_ms: Option<u32>,
    cpu_time_us: Option<u64>,
    memory_growth_kib: Option<u64>,
}

impl PresentationTelemetry {
    pub const fn new(
        target: PresentationEvidenceTarget,
        path: PresentationPath,
        workload: PresentationWorkload,
    ) -> Self {
        Self {
            target,
            path,
            workload,
            frames_requested: 0,
            frames_presented: 0,
            dropped_frames: 0,
            page_flip_events: 0,
            frame_callbacks_sent: 0,
            damage_commits: 0,
            full_frame_readbacks: 0,
            cpu_framebuffer_copies: 0,
            settled_idle_observations: 0,
            settled_idle_repaints: 0,
            repeating_repaint_timer_after_settle: false,
            max_frame_time_us: None,
            max_input_to_present_us: None,
            input_to_present_samples: 0,
            observation_window_ms: None,
            cpu_time_us: None,
            memory_growth_kib: None,
        }
    }

    pub fn record_frame_requested(&mut self) -> Result<(), PresentationTelemetryError> {
        increment_event(&mut self.frames_requested, "frame-request")
    }

    pub fn record_page_flip(
        &mut self,
        frame_time_us: u32,
    ) -> Result<(), PresentationTelemetryError> {
        if frame_time_us == 0 {
            return Err(PresentationTelemetryError::InvalidFrameTime);
        }
        self.require_outstanding_frame()?;
        increment_event(&mut self.frames_presented, "frame-presented")?;
        increment_event(&mut self.page_flip_events, "page-flip")?;
        self.max_frame_time_us = Some(
            self.max_frame_time_us
                .map_or(frame_time_us, |current| current.max(frame_time_us)),
        );
        Ok(())
    }

    pub fn record_dropped_frame(&mut self) -> Result<(), PresentationTelemetryError> {
        self.require_outstanding_frame()?;
        increment_event(&mut self.dropped_frames, "dropped-frame")
    }

    pub fn record_frame_callbacks(&mut self, count: u32) -> Result<(), PresentationTelemetryError> {
        add_events(&mut self.frame_callbacks_sent, count, "frame-callback")
    }

    pub fn record_damage_commits(&mut self, count: u32) -> Result<(), PresentationTelemetryError> {
        add_events(&mut self.damage_commits, count, "damage-commit")
    }

    pub fn record_full_frame_readback(&mut self) -> Result<(), PresentationTelemetryError> {
        increment_event(&mut self.full_frame_readbacks, "full-frame-readback")
    }

    pub fn record_cpu_framebuffer_copy(&mut self) -> Result<(), PresentationTelemetryError> {
        increment_event(&mut self.cpu_framebuffer_copies, "cpu-framebuffer-copy")
    }

    pub fn record_settled_idle_repaint(&mut self) -> Result<(), PresentationTelemetryError> {
        increment_event(&mut self.settled_idle_repaints, "settled-idle-repaint")
    }

    pub fn record_settled_idle_observation(&mut self) -> Result<(), PresentationTelemetryError> {
        increment_event(
            &mut self.settled_idle_observations,
            "settled-idle-observation",
        )
    }

    pub const fn mark_repeating_repaint_timer_after_settle(&mut self) {
        self.repeating_repaint_timer_after_settle = true;
    }

    pub fn record_input_to_present(
        &mut self,
        latency_us: u32,
    ) -> Result<(), PresentationTelemetryError> {
        if latency_us == 0 {
            return Err(PresentationTelemetryError::InvalidInputToPresentTime);
        }
        increment_event(
            &mut self.input_to_present_samples,
            "input-to-present-sample",
        )?;
        self.max_input_to_present_us = Some(
            self.max_input_to_present_us
                .map_or(latency_us, |current| current.max(latency_us)),
        );
        Ok(())
    }

    pub fn record_resource_observation(
        &mut self,
        observation_window_ms: u32,
        cpu_time_us: u64,
        memory_growth_kib: u64,
    ) -> Result<(), PresentationTelemetryError> {
        if observation_window_ms == 0 {
            return Err(PresentationTelemetryError::InvalidObservationWindow);
        }
        self.observation_window_ms = Some(observation_window_ms);
        self.cpu_time_us = Some(cpu_time_us);
        self.memory_growth_kib = Some(memory_growth_kib);
        Ok(())
    }

    pub fn finish(
        mut self,
        observation_window_ms: u32,
        cpu_time_us: u64,
        memory_growth_kib: u64,
    ) -> Result<PresentationSample, PresentationTelemetryError> {
        self.record_resource_observation(observation_window_ms, cpu_time_us, memory_growth_kib)?;
        if self.frames_requested == 0
            || self.frames_presented.checked_add(self.dropped_frames) != Some(self.frames_requested)
        {
            return Err(PresentationTelemetryError::IncompleteFrameAccounting);
        }
        Ok(PresentationSample {
            target: self.target,
            path: self.path,
            workload: self.workload,
            observation_window_ms: self.observation_window_ms.unwrap_or_default(),
            frames_requested: self.frames_requested,
            frames_presented: self.frames_presented,
            dropped_frames: self.dropped_frames,
            page_flip_events: self.page_flip_events,
            frame_callbacks_sent: self.frame_callbacks_sent,
            damage_commits: self.damage_commits,
            full_frame_readbacks: self.full_frame_readbacks,
            cpu_framebuffer_copies: self.cpu_framebuffer_copies,
            settled_idle_observations: self.settled_idle_observations,
            settled_idle_repaints: self.settled_idle_repaints,
            repeating_repaint_timer_after_settle: self.repeating_repaint_timer_after_settle,
            max_frame_time_us: self.max_frame_time_us,
            max_input_to_present_us: self.max_input_to_present_us,
            cpu_time_us: self.cpu_time_us,
            memory_growth_kib: self.memory_growth_kib,
        })
    }

    pub const fn event_snapshot(&self) -> PresentationEventSnapshot {
        PresentationEventSnapshot {
            target: self.target,
            path: self.path,
            workload: self.workload,
            frames_requested: self.frames_requested,
            frames_presented: self.frames_presented,
            page_flip_events: self.page_flip_events,
            frame_callbacks_sent: self.frame_callbacks_sent,
            damage_commits: self.damage_commits,
            full_frame_readbacks: self.full_frame_readbacks,
            input_to_present_samples: self.input_to_present_samples,
            settled_idle_observations: self.settled_idle_observations,
            settled_idle_repaints: self.settled_idle_repaints,
            repeating_repaint_timer_after_settle: self.repeating_repaint_timer_after_settle,
            cpu_framebuffer_copies: self.cpu_framebuffer_copies,
            max_frame_time_us: self.max_frame_time_us,
            max_input_to_present_us: self.max_input_to_present_us,
            observation_window_ms: self.observation_window_ms,
            cpu_time_us: self.cpu_time_us,
            memory_growth_kib: self.memory_growth_kib,
        }
    }

    fn require_outstanding_frame(&self) -> Result<(), PresentationTelemetryError> {
        let accounted = self
            .frames_presented
            .checked_add(self.dropped_frames)
            .ok_or(PresentationTelemetryError::EventLimitExceeded(
                "frame-accounting",
            ))?;
        if accounted >= self.frames_requested {
            return Err(PresentationTelemetryError::NoOutstandingFrame);
        }
        Ok(())
    }
}

fn increment_event(value: &mut u32, event: &'static str) -> Result<(), PresentationTelemetryError> {
    add_events(value, 1, event)
}

fn add_events(
    value: &mut u32,
    count: u32,
    event: &'static str,
) -> Result<(), PresentationTelemetryError> {
    let next = value
        .checked_add(count)
        .filter(|next| *next <= MAX_PRESENTATION_EVENTS)
        .ok_or(PresentationTelemetryError::EventLimitExceeded(event))?;
    *value = next;
    Ok(())
}

impl PresentationSample {
    fn frame_accounting_ready(self) -> bool {
        self.observation_window_ms > 0
            && self.frames_requested > 0
            && self.frames_presented > 0
            && self.frames_presented.checked_add(self.dropped_frames) == Some(self.frames_requested)
    }

    fn event_scheduling_ready(self) -> bool {
        self.page_flip_events == self.frames_presented
            && (self.workload != PresentationWorkload::MultiClient || self.frame_callbacks_sent > 0)
    }

    fn production_path_ready(self) -> bool {
        self.path == PresentationPath::ProductionGbmKms
            && self.full_frame_readbacks == 0
            && self.cpu_framebuffer_copies == 0
    }

    fn damage_ready(self) -> bool {
        match self.workload {
            PresentationWorkload::Idle => self.damage_commits <= 1,
            PresentationWorkload::MultiClient => self.damage_commits > 0,
            PresentationWorkload::WindowInteraction | PresentationWorkload::Animation => true,
        }
    }

    fn idle_suppression_ready(self) -> bool {
        let frame_count_ready = self.workload != PresentationWorkload::Idle
            || (self.frames_requested == 1 && self.frames_presented == 1);
        frame_count_ready
            && (self.workload != PresentationWorkload::Idle || self.settled_idle_observations > 0)
            && self.settled_idle_repaints == 0
            && !self.repeating_repaint_timer_after_settle
    }

    fn timings_ready(self, budget: PresentationBudget) -> bool {
        self.max_frame_time_us
            .is_some_and(|value| value <= budget.max_frame_time_us)
            && (self.workload == PresentationWorkload::Idle
                || self
                    .max_input_to_present_us
                    .is_some_and(|value| value <= budget.max_input_to_present_us))
    }

    fn resources_ready(self, budget: PresentationBudget) -> bool {
        self.cpu_time_us
            .is_some_and(|value| value <= budget.max_cpu_time_us)
            && self
                .memory_growth_kib
                .is_some_and(|value| value <= budget.max_memory_growth_kib)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticReadbackEvidence {
    pub target: PresentationEvidenceTarget,
    pub path: PresentationPath,
    pub captured_frames: u32,
    pub full_frame_readbacks: u32,
    pub production_frames_read_back: u32,
    pub production_frames_blocked: u32,
    pub kms_activated: bool,
    pub display_output_started: bool,
}

impl DiagnosticReadbackEvidence {
    pub fn is_isolated(self, target: PresentationEvidenceTarget) -> bool {
        self.target == target
            && self.path == PresentationPath::DiagnosticReadback
            && self.captured_frames > 0
            && self.full_frame_readbacks > 0
            && self.full_frame_readbacks <= self.captured_frames
            && self.production_frames_read_back == 0
            && self.production_frames_blocked == 0
            && !self.kms_activated
            && !self.display_output_started
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct R2PresentationReport {
    pub target: PresentationEvidenceTarget,
    pub sample_count_bounded: bool,
    pub workload_coverage_ready: bool,
    pub production_path_ready: bool,
    pub frame_accounting_ready: bool,
    pub event_scheduling_ready: bool,
    pub damage_ready: bool,
    pub idle_suppression_ready: bool,
    pub timings_ready: bool,
    pub resources_ready: bool,
    pub dropped_frames_ready: bool,
    pub diagnostic_readback_isolated: bool,
}

impl R2PresentationReport {
    pub fn evaluate(
        target: PresentationEvidenceTarget,
        budget: PresentationBudget,
        samples: &[PresentationSample],
        diagnostic: DiagnosticReadbackEvidence,
    ) -> Self {
        let sample_count_bounded = !samples.is_empty() && samples.len() <= MAX_PRESENTATION_SAMPLES;
        let targets_match = samples.iter().all(|sample| sample.target == target);
        let workload_coverage_ready = PresentationWorkload::ALL.iter().all(|workload| {
            samples
                .iter()
                .filter(|sample| sample.workload == *workload)
                .count()
                == 1
        });

        Self {
            target,
            sample_count_bounded,
            workload_coverage_ready,
            production_path_ready: targets_match
                && samples
                    .iter()
                    .copied()
                    .all(PresentationSample::production_path_ready),
            frame_accounting_ready: samples
                .iter()
                .copied()
                .all(PresentationSample::frame_accounting_ready),
            event_scheduling_ready: samples
                .iter()
                .copied()
                .all(PresentationSample::event_scheduling_ready),
            damage_ready: samples
                .iter()
                .copied()
                .all(PresentationSample::damage_ready),
            idle_suppression_ready: samples
                .iter()
                .copied()
                .all(PresentationSample::idle_suppression_ready),
            timings_ready: budget.is_bounded()
                && samples
                    .iter()
                    .copied()
                    .all(|sample| sample.timings_ready(budget)),
            resources_ready: budget.is_bounded()
                && samples
                    .iter()
                    .copied()
                    .all(|sample| sample.resources_ready(budget)),
            dropped_frames_ready: samples
                .iter()
                .all(|sample| sample.dropped_frames <= budget.max_dropped_frames),
            diagnostic_readback_isolated: diagnostic.is_isolated(target),
        }
    }

    pub const fn is_baseline_ready(self) -> bool {
        self.sample_count_bounded
            && self.workload_coverage_ready
            && self.production_path_ready
            && self.frame_accounting_ready
            && self.event_scheduling_ready
            && self.damage_ready
            && self.idle_suppression_ready
            && self.timings_ready
            && self.resources_ready
            && self.dropped_frames_ready
            && self.diagnostic_readback_isolated
    }

    pub const fn supports_release_claim(self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUDGET: PresentationBudget = PresentationBudget {
        max_frame_time_us: 20_000,
        max_input_to_present_us: 50_000,
        max_dropped_frames: 1,
        max_cpu_time_us: 500_000,
        max_memory_growth_kib: 4_096,
    };

    fn sample(workload: PresentationWorkload) -> PresentationSample {
        let idle = workload == PresentationWorkload::Idle;
        let frame_count = if idle { 1 } else { 60 };
        PresentationSample {
            target: PresentationEvidenceTarget::HostFixture,
            path: PresentationPath::ProductionGbmKms,
            workload,
            observation_window_ms: 1_000,
            frames_requested: frame_count,
            frames_presented: frame_count,
            dropped_frames: 0,
            page_flip_events: frame_count,
            frame_callbacks_sent: if idle { 0 } else { frame_count },
            damage_commits: usize::from(workload != PresentationWorkload::Idle) as u32 * 12,
            full_frame_readbacks: 0,
            cpu_framebuffer_copies: 0,
            settled_idle_observations: u32::from(idle) * 5,
            settled_idle_repaints: 0,
            repeating_repaint_timer_after_settle: false,
            max_frame_time_us: Some(16_700),
            max_input_to_present_us: (workload != PresentationWorkload::Idle).then_some(28_000),
            cpu_time_us: Some(120_000),
            memory_growth_kib: Some(256),
        }
    }

    fn diagnostic() -> DiagnosticReadbackEvidence {
        DiagnosticReadbackEvidence {
            target: PresentationEvidenceTarget::HostFixture,
            path: PresentationPath::DiagnosticReadback,
            captured_frames: 2,
            full_frame_readbacks: 2,
            production_frames_read_back: 0,
            production_frames_blocked: 0,
            kms_activated: false,
            display_output_started: false,
        }
    }

    fn samples() -> [PresentationSample; 4] {
        PresentationWorkload::ALL.map(sample)
    }

    #[test]
    fn telemetry_builds_an_ordered_bounded_sample_from_live_event_inputs() {
        let mut telemetry = PresentationTelemetry::new(
            PresentationEvidenceTarget::HostFixture,
            PresentationPath::ProductionGbmKms,
            PresentationWorkload::WindowInteraction,
        );
        telemetry.record_frame_requested().unwrap();
        telemetry.record_damage_commits(1).unwrap();
        telemetry.record_frame_callbacks(2).unwrap();
        telemetry.record_page_flip(14_000).unwrap();
        telemetry.record_input_to_present(24_000).unwrap();
        let event_snapshot = telemetry.event_snapshot();
        assert_eq!(event_snapshot.frames_requested, 1);
        assert_eq!(event_snapshot.frames_presented, 1);
        assert_eq!(event_snapshot.page_flip_events, 1);
        assert_eq!(event_snapshot.frame_callbacks_sent, 2);
        assert_eq!(event_snapshot.damage_commits, 1);
        assert_eq!(event_snapshot.max_frame_time_us, Some(14_000));
        assert_eq!(event_snapshot.input_to_present_samples, 1);
        assert_eq!(event_snapshot.max_input_to_present_us, Some(24_000));
        telemetry.record_settled_idle_observation().unwrap();
        let idle_snapshot = telemetry.event_snapshot();
        assert_eq!(idle_snapshot.settled_idle_observations, 1);
        assert_eq!(idle_snapshot.settled_idle_repaints, 0);
        assert!(!idle_snapshot.repeating_repaint_timer_after_settle);
        telemetry
            .record_resource_observation(1_000, 90_000, 128)
            .unwrap();
        let resource_snapshot = telemetry.event_snapshot();
        assert_eq!(resource_snapshot.observation_window_ms, Some(1_000));
        assert_eq!(resource_snapshot.cpu_time_us, Some(90_000));
        assert_eq!(resource_snapshot.memory_growth_kib, Some(128));
        telemetry.record_frame_requested().unwrap();
        telemetry.record_dropped_frame().unwrap();

        let collected = telemetry.finish(1_000, 90_000, 128).unwrap();
        assert_eq!(collected.frames_requested, 2);
        assert_eq!(collected.frames_presented, 1);
        assert_eq!(collected.dropped_frames, 1);
        assert_eq!(collected.page_flip_events, 1);
        assert_eq!(collected.frame_callbacks_sent, 2);
        assert_eq!(collected.max_frame_time_us, Some(14_000));
        assert_eq!(collected.max_input_to_present_us, Some(24_000));
        assert_eq!(collected.cpu_time_us, Some(90_000));
        assert_eq!(collected.memory_growth_kib, Some(128));
    }

    #[test]
    fn telemetry_rejects_out_of_order_incomplete_and_unbounded_events_atomically() {
        let mut telemetry = PresentationTelemetry::new(
            PresentationEvidenceTarget::HostFixture,
            PresentationPath::ProductionGbmKms,
            PresentationWorkload::Animation,
        );
        assert_eq!(
            telemetry.record_page_flip(10_000),
            Err(PresentationTelemetryError::NoOutstandingFrame)
        );
        assert_eq!(
            telemetry.record_frame_callbacks(MAX_PRESENTATION_EVENTS + 1),
            Err(PresentationTelemetryError::EventLimitExceeded(
                "frame-callback"
            ))
        );
        assert_eq!(telemetry.frame_callbacks_sent, 0);
        telemetry.record_frame_requested().unwrap();
        assert_eq!(
            telemetry.clone().finish(1_000, 1, 0),
            Err(PresentationTelemetryError::IncompleteFrameAccounting)
        );
        assert_eq!(
            telemetry.record_page_flip(0),
            Err(PresentationTelemetryError::InvalidFrameTime)
        );
        telemetry.record_dropped_frame().unwrap();
        assert_eq!(
            telemetry.finish(0, 1, 0),
            Err(PresentationTelemetryError::InvalidObservationWindow)
        );
    }

    #[test]
    fn legacy_cpu_copy_telemetry_cannot_satisfy_the_production_report() {
        let mut telemetry = PresentationTelemetry::new(
            PresentationEvidenceTarget::HostFixture,
            PresentationPath::LegacyCpuCopy,
            PresentationWorkload::WindowInteraction,
        );
        telemetry.record_frame_requested().unwrap();
        telemetry.record_damage_commits(1).unwrap();
        telemetry.record_frame_callbacks(1).unwrap();
        telemetry.record_cpu_framebuffer_copy().unwrap();
        telemetry.record_full_frame_readback().unwrap();
        telemetry.record_page_flip(10_000).unwrap();
        telemetry.record_input_to_present(20_000).unwrap();
        let mut fixtures = samples();
        fixtures[1] = telemetry.finish(1_000, 80_000, 64).unwrap();

        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &fixtures,
            diagnostic(),
        );
        assert!(!report.production_path_ready);
        assert!(!report.is_baseline_ready());
    }

    #[test]
    fn complete_bounded_fixture_baseline_is_accepted_without_release_claim() {
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &samples(),
            diagnostic(),
        );

        assert!(report.is_baseline_ready());
        assert!(!report.supports_release_claim());
    }

    #[test]
    fn qemu_tcg_bochs_v1_budget_is_bounded_and_not_a_release_claim() {
        assert!(QEMU_TCG_BOCHS_V1_BUDGET.is_bounded());
        assert_eq!(QEMU_TCG_BOCHS_V1_BUDGET.max_frame_time_us, 50_000);
        assert_eq!(QEMU_TCG_BOCHS_V1_BUDGET.max_input_to_present_us, 60_000_000);
        assert_eq!(QEMU_TCG_BOCHS_V1_BUDGET.max_dropped_frames, 0);
        assert_eq!(QEMU_TCG_BOCHS_V1_BUDGET.max_cpu_time_us, 180_000_000);
        assert_eq!(QEMU_TCG_BOCHS_V1_BUDGET.max_memory_growth_kib, 163_840);
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            QEMU_TCG_BOCHS_V1_BUDGET,
            &samples(),
            diagnostic(),
        );
        assert!(report.is_baseline_ready());
        assert!(!report.supports_release_claim());
    }

    #[test]
    fn qemu_tcg_bochs_soak_v1_budget_is_bounded_and_not_a_release_claim() {
        assert!(QEMU_TCG_BOCHS_SOAK_V1_BUDGET.is_bounded());
        assert_eq!(QEMU_TCG_BOCHS_SOAK_V1_BUDGET.max_frame_time_us, 50_000);
        assert_eq!(
            QEMU_TCG_BOCHS_SOAK_V1_BUDGET.max_input_to_present_us,
            60_000_000
        );
        assert_eq!(QEMU_TCG_BOCHS_SOAK_V1_BUDGET.max_dropped_frames, 0);
        assert_eq!(QEMU_TCG_BOCHS_SOAK_V1_BUDGET.max_cpu_time_us, 720_000_000);
        assert_eq!(QEMU_TCG_BOCHS_SOAK_V1_BUDGET.max_memory_growth_kib, 163_840);
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            QEMU_TCG_BOCHS_SOAK_V1_BUDGET,
            &samples(),
            diagnostic(),
        );
        assert!(report.is_baseline_ready());
        assert!(!report.supports_release_claim());
    }

    #[test]
    fn shell_workloads_do_not_require_client_callbacks_or_damage() {
        let mut fixtures = samples();
        for sample in &mut fixtures[1..3] {
            sample.frame_callbacks_sent = 0;
            sample.damage_commits = 0;
        }
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &fixtures,
            diagnostic(),
        );
        assert!(report.event_scheduling_ready);
        assert!(report.damage_ready);

        fixtures[3].frame_callbacks_sent = 0;
        fixtures[3].damage_commits = 0;
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &fixtures,
            diagnostic(),
        );
        assert!(!report.event_scheduling_ready);
        assert!(!report.damage_ready);
    }

    #[test]
    fn production_readback_copy_and_unbounded_idle_repaint_fail_closed() {
        let mut fixtures = samples();
        fixtures[0].full_frame_readbacks = 1;
        fixtures[0].cpu_framebuffer_copies = 1;
        fixtures[0].settled_idle_repaints = 1;
        fixtures[0].repeating_repaint_timer_after_settle = true;
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &fixtures,
            diagnostic(),
        );

        assert!(!report.production_path_ready);
        assert!(!report.idle_suppression_ready);
        assert!(!report.is_baseline_ready());
    }

    #[test]
    fn idle_workload_requires_a_real_settled_observation() {
        let mut fixtures = samples();
        fixtures[0].settled_idle_observations = 0;
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &fixtures,
            diagnostic(),
        );

        assert!(!report.idle_suppression_ready);
        assert!(!report.is_baseline_ready());
    }

    #[test]
    fn missing_workload_and_mismatched_frame_events_fail_closed() {
        let mut fixtures = samples();
        fixtures[3].workload = PresentationWorkload::Animation;
        fixtures[1].page_flip_events = 59;
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &fixtures,
            diagnostic(),
        );

        assert!(!report.workload_coverage_ready);
        assert!(!report.event_scheduling_ready);
        assert!(!report.is_baseline_ready());
    }

    #[test]
    fn timing_resource_drop_and_diagnostic_leak_fail_closed() {
        let mut fixtures = samples();
        fixtures[1].max_input_to_present_us = Some(50_001);
        fixtures[2].cpu_time_us = Some(500_001);
        fixtures[3].dropped_frames = 2;
        fixtures[3].frames_presented = 58;
        let mut readback = diagnostic();
        readback.production_frames_read_back = 1;
        let report = R2PresentationReport::evaluate(
            PresentationEvidenceTarget::HostFixture,
            BUDGET,
            &fixtures,
            readback,
        );

        assert!(!report.timings_ready);
        assert!(!report.resources_ready);
        assert!(!report.dropped_frames_ready);
        assert!(!report.diagnostic_readback_isolated);
        assert!(!report.is_baseline_ready());
    }

    #[test]
    fn diagnostic_readback_must_use_an_offscreen_non_kms_path() {
        let mut readback = diagnostic();
        readback.path = PresentationPath::ProductionGbmKms;
        assert!(!readback.is_isolated(PresentationEvidenceTarget::HostFixture));

        readback = diagnostic();
        readback.kms_activated = true;
        assert!(!readback.is_isolated(PresentationEvidenceTarget::HostFixture));

        readback = diagnostic();
        readback.display_output_started = true;
        assert!(!readback.is_isolated(PresentationEvidenceTarget::HostFixture));
    }
}
