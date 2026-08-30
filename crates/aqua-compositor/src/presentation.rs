pub const MAX_PRESENTATION_SAMPLES: usize = 8;

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
        self.max_frame_time_us > 0 && self.max_input_to_present_us > 0 && self.max_cpu_time_us > 0
    }
}

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
    pub settled_idle_repaints: u32,
    pub repeating_repaint_timer_after_settle: bool,
    pub max_frame_time_us: Option<u32>,
    pub max_input_to_present_us: Option<u32>,
    pub cpu_time_us: Option<u64>,
    pub memory_growth_kib: Option<u64>,
}

impl PresentationSample {
    fn frame_accounting_ready(self) -> bool {
        self.observation_window_ms > 0
            && self.frames_requested > 0
            && self.frames_presented > 0
            && self.frames_presented.checked_add(self.dropped_frames) == Some(self.frames_requested)
    }

    fn event_scheduling_ready(self) -> bool {
        self.page_flip_events == self.frames_presented && self.frame_callbacks_sent > 0
    }

    fn production_path_ready(self) -> bool {
        self.path == PresentationPath::ProductionGbmKms
            && self.full_frame_readbacks == 0
            && self.cpu_framebuffer_copies == 0
    }

    fn damage_ready(self) -> bool {
        match self.workload {
            PresentationWorkload::Idle => self.damage_commits <= 1,
            _ => self.damage_commits > 0,
        }
    }

    fn idle_suppression_ready(self) -> bool {
        let frame_count_ready = self.workload != PresentationWorkload::Idle
            || (self.frames_requested == 1 && self.frames_presented == 1);
        frame_count_ready
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
    pub captured_frames: u32,
    pub full_frame_readbacks: u32,
    pub production_frames_read_back: u32,
    pub production_frames_blocked: u32,
}

impl DiagnosticReadbackEvidence {
    pub fn is_isolated(self, target: PresentationEvidenceTarget) -> bool {
        self.target == target
            && self.captured_frames > 0
            && self.full_frame_readbacks > 0
            && self.full_frame_readbacks <= self.captured_frames
            && self.production_frames_read_back == 0
            && self.production_frames_blocked == 0
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
            frame_callbacks_sent: frame_count,
            damage_commits: usize::from(workload != PresentationWorkload::Idle) as u32 * 12,
            full_frame_readbacks: 0,
            cpu_framebuffer_copies: 0,
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
            captured_frames: 2,
            full_frame_readbacks: 2,
            production_frames_read_back: 0,
            production_frames_blocked: 0,
        }
    }

    fn samples() -> [PresentationSample; 4] {
        PresentationWorkload::ALL.map(sample)
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
}
