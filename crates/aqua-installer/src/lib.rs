use aqua_scene::{Rect, Viewport};
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const INSTALLER_STATUS: &str = "installer-state-model-ready";
pub const INSTALLER_UI_STATUS: &str = "keyboard-navigable-installer-window-contract-ready";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerStep {
    Welcome,
    Language,
    Keyboard,
    Partitions,
    TimeZone,
    UserInformation,
    Summary,
    Installation,
    Completed,
}

impl InstallerStep {
    pub const ALL: [Self; 9] = [
        Self::Welcome,
        Self::Language,
        Self::Keyboard,
        Self::Partitions,
        Self::TimeZone,
        Self::UserInformation,
        Self::Summary,
        Self::Installation,
        Self::Completed,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Welcome => "welcome",
            Self::Language => "language",
            Self::Keyboard => "keyboard",
            Self::Partitions => "partitions",
            Self::TimeZone => "time-zone",
            Self::UserInformation => "user-information",
            Self::Summary => "summary",
            Self::Installation => "installation",
            Self::Completed => "completed",
        }
    }

    pub const fn label_tr(self) -> &'static str {
        match self {
            Self::Welcome => "Hoş Geldiniz",
            Self::Language => "Dil",
            Self::Keyboard => "Klavye",
            Self::Partitions => "Bölümler",
            Self::TimeZone => "Zaman Dilimi",
            Self::UserInformation => "Kullanıcı Bilgisi",
            Self::Summary => "Özet",
            Self::Installation => "Kurulum",
            Self::Completed => "Tamamlandı",
        }
    }

    const fn next(self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::Language),
            Self::Language => Some(Self::Keyboard),
            Self::Keyboard => Some(Self::Partitions),
            Self::Partitions => Some(Self::TimeZone),
            Self::TimeZone => Some(Self::UserInformation),
            Self::UserInformation => Some(Self::Summary),
            Self::Summary | Self::Installation | Self::Completed => None,
        }
    }

    const fn previous(self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::Language => Some(Self::Welcome),
            Self::Keyboard => Some(Self::Language),
            Self::Partitions => Some(Self::Keyboard),
            Self::TimeZone => Some(Self::Partitions),
            Self::UserInformation => Some(Self::TimeZone),
            Self::Summary => Some(Self::UserInformation),
            Self::Installation | Self::Completed => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerUiMaterial {
    WindowIceSurface,
    RailFrostedSurface,
    ContentClearSurface,
    FooterFrostedSurface,
    PrimaryControl,
    SecondaryControl,
}

impl InstallerUiMaterial {
    pub const fn id(self) -> &'static str {
        match self {
            Self::WindowIceSurface => "window-ice-surface",
            Self::RailFrostedSurface => "rail-frosted-surface",
            Self::ContentClearSurface => "content-clear-surface",
            Self::FooterFrostedSurface => "footer-frosted-surface",
            Self::PrimaryControl => "primary-control",
            Self::SecondaryControl => "secondary-control",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallerUiSurface {
    pub id: &'static str,
    pub rect: Rect,
    pub material: InstallerUiMaterial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerWindowLayout {
    pub viewport: Viewport,
    pub window: Rect,
    pub titlebar: Rect,
    pub step_rail: Rect,
    pub content: Rect,
    pub footer: Rect,
    pub language_control: Rect,
    pub cancel_button: Rect,
    pub back_button: Rect,
    pub forward_button: Rect,
    pub progress_track: Rect,
}

impl InstallerWindowLayout {
    pub fn for_viewport(viewport: Viewport) -> Result<Self, InstallerUiLayoutError> {
        if !viewport.is_supported() {
            return Err(InstallerUiLayoutError::UnsupportedViewport(viewport));
        }

        let margin = if viewport.width >= 1200 { 32 } else { 20 };
        let window = Rect {
            x: margin,
            y: margin,
            width: viewport.width - margin * 2,
            height: viewport.height - margin * 2,
        };
        let titlebar_height = if viewport.height >= 800 { 60 } else { 52 };
        let footer_height = if viewport.height >= 800 { 96 } else { 82 };
        let rail_width = if viewport.width >= 1200 { 264 } else { 196 };
        let control_height = 48;
        let control_y = window.bottom() - footer_height + (footer_height - control_height) / 2;
        let side_padding = if viewport.width >= 1200 { 32 } else { 20 };
        let primary_width = if viewport.width >= 1200 { 160 } else { 128 };
        let secondary_width = if viewport.width >= 1200 { 120 } else { 96 };
        let control_gap = 16;
        let forward_button = Rect {
            x: window.right() - side_padding - primary_width,
            y: control_y,
            width: primary_width,
            height: control_height,
        };
        let back_button = Rect {
            x: forward_button.x - control_gap - secondary_width,
            y: control_y,
            width: secondary_width,
            height: control_height,
        };
        let cancel_button = Rect {
            x: back_button.x - control_gap - secondary_width,
            y: control_y,
            width: secondary_width,
            height: control_height,
        };
        let language_control = Rect {
            x: window.x + side_padding,
            y: control_y,
            width: if viewport.width >= 1200 { 164 } else { 132 },
            height: control_height,
        };
        let content_top = window.y + titlebar_height;
        let content_height = window.height - titlebar_height - footer_height;
        let content = Rect {
            x: window.x + rail_width,
            y: content_top,
            width: window.width - rail_width,
            height: content_height,
        };
        let progress_margin = if viewport.width >= 1200 { 72 } else { 40 };

        Ok(Self {
            viewport,
            window,
            titlebar: Rect {
                x: window.x,
                y: window.y,
                width: window.width,
                height: titlebar_height,
            },
            step_rail: Rect {
                x: window.x,
                y: content_top,
                width: rail_width,
                height: content_height,
            },
            content,
            footer: Rect {
                x: window.x,
                y: window.bottom() - footer_height,
                width: window.width,
                height: footer_height,
            },
            language_control,
            cancel_button,
            back_button,
            forward_button,
            progress_track: Rect {
                x: content.x + progress_margin,
                y: content.bottom() - 72,
                width: content.width - progress_margin * 2,
                height: 8,
            },
        })
    }

    pub fn surfaces(&self) -> [InstallerUiSurface; 6] {
        [
            InstallerUiSurface {
                id: "installer-window",
                rect: self.window,
                material: InstallerUiMaterial::WindowIceSurface,
            },
            InstallerUiSurface {
                id: "installer-step-rail",
                rect: self.step_rail,
                material: InstallerUiMaterial::RailFrostedSurface,
            },
            InstallerUiSurface {
                id: "installer-content",
                rect: self.content,
                material: InstallerUiMaterial::ContentClearSurface,
            },
            InstallerUiSurface {
                id: "installer-footer",
                rect: self.footer,
                material: InstallerUiMaterial::FooterFrostedSurface,
            },
            InstallerUiSurface {
                id: "installer-back",
                rect: self.back_button,
                material: InstallerUiMaterial::SecondaryControl,
            },
            InstallerUiSurface {
                id: "installer-forward",
                rect: self.forward_button,
                material: InstallerUiMaterial::PrimaryControl,
            },
        ]
    }

    pub fn fits_viewport(&self) -> bool {
        self.window.fits_in(self.viewport)
            && self
                .surfaces()
                .iter()
                .all(|surface| surface.rect.fits_in(self.viewport))
            && [
                self.titlebar,
                self.language_control,
                self.cancel_button,
                self.progress_track,
            ]
            .iter()
            .all(|rect| rect.fits_in(self.viewport))
    }

    pub fn regions_are_separated(&self) -> bool {
        !self.titlebar.overlaps(self.footer)
            && !self.step_rail.overlaps(self.footer)
            && !self.content.overlaps(self.footer)
            && !self.step_rail.overlaps(self.content)
            && !self.language_control.overlaps(self.cancel_button)
            && !self.cancel_button.overlaps(self.back_button)
            && !self.back_button.overlaps(self.forward_button)
    }

    pub const fn content_padding(&self) -> u32 {
        if self.viewport.width >= 1200 {
            64
        } else {
            32
        }
    }

    pub const fn content_heading_y(&self) -> u32 {
        self.content.y + if self.viewport.height >= 800 { 112 } else { 70 }
    }

    pub const fn content_row_width(&self) -> u32 {
        self.content
            .width
            .saturating_sub(self.content_padding() * 2)
    }

    pub const fn choice_row(&self, index: usize) -> Rect {
        Rect {
            x: self.content.x + self.content_padding(),
            y: self.content_heading_y() + 78 + index as u32 * 78,
            width: self.content_row_width(),
            height: 66,
        }
    }

    pub const fn disk_row(&self, index: usize) -> Rect {
        Rect {
            x: self.content.x + self.content_padding(),
            y: self.content_heading_y() + 72 + index as u32 * 68,
            width: self.content_row_width(),
            height: 58,
        }
    }

    pub const fn user_field_row(&self, field: InstallerUserField) -> Rect {
        let index = match field {
            InstallerUserField::Username => 0,
            InstallerUserField::DisplayName => 1,
            InstallerUserField::Password => 2,
        };
        Rect {
            x: self.content.x + self.content_padding(),
            y: self.content_heading_y() + 76 + index * 80,
            width: self.content_row_width(),
            height: 68,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerUiLayoutError {
    UnsupportedViewport(Viewport),
}

impl fmt::Display for InstallerUiLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedViewport(viewport) => write!(
                formatter,
                "unsupported installer viewport {}x{}",
                viewport.width, viewport.height
            ),
        }
    }
}

impl Error for InstallerUiLayoutError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerFocusTarget {
    StepContent,
    LanguageControl,
    Cancel,
    Back,
    Forward,
    ProgressStatus,
    Finish,
}

impl InstallerFocusTarget {
    pub const fn id(self) -> &'static str {
        match self {
            Self::StepContent => "step-content",
            Self::LanguageControl => "language-control",
            Self::Cancel => "cancel",
            Self::Back => "back",
            Self::Forward => "forward",
            Self::ProgressStatus => "progress-status",
            Self::Finish => "finish",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerUiKey {
    Tab,
    BackTab,
    Left,
    Right,
    Home,
    End,
    Activate,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerUiAction {
    None,
    FocusChanged(InstallerFocusTarget),
    ActivateStepContent(InstallerStep),
    OpenLanguageControl,
    CancelRequested,
    RetreatRequested,
    AdvanceRequested,
    BeginInstallRequested,
    FinishRequested,
}

impl InstallerUiAction {
    pub const fn changed(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerUiState {
    step: InstallerStep,
    focus: InstallerFocusTarget,
}

impl InstallerUiState {
    pub fn new(model: &InstallerModel) -> Self {
        let targets = installer_focus_order(model.step());
        Self {
            step: model.step(),
            focus: targets[0],
        }
    }

    pub const fn step(&self) -> InstallerStep {
        self.step
    }

    pub const fn focus(&self) -> InstallerFocusTarget {
        self.focus
    }

    pub fn sync_step(&mut self, model: &InstallerModel) -> bool {
        if self.step == model.step() {
            return false;
        }
        self.step = model.step();
        self.focus = installer_focus_order(self.step)[0];
        true
    }

    pub fn handle_key(&mut self, key: InstallerUiKey) -> InstallerUiAction {
        let targets = installer_focus_order(self.step);
        match key {
            InstallerUiKey::Tab | InstallerUiKey::Right => self.move_focus(targets, 1),
            InstallerUiKey::BackTab | InstallerUiKey::Left => self.move_focus(targets, -1),
            InstallerUiKey::Home => self.set_focus(targets[0]),
            InstallerUiKey::End => self.set_focus(targets[targets.len() - 1]),
            InstallerUiKey::Escape if targets.contains(&InstallerFocusTarget::Cancel) => {
                InstallerUiAction::CancelRequested
            }
            InstallerUiKey::Escape => InstallerUiAction::None,
            InstallerUiKey::Activate => match self.focus {
                InstallerFocusTarget::StepContent => {
                    InstallerUiAction::ActivateStepContent(self.step)
                }
                InstallerFocusTarget::LanguageControl => InstallerUiAction::OpenLanguageControl,
                InstallerFocusTarget::Cancel => InstallerUiAction::CancelRequested,
                InstallerFocusTarget::Back => InstallerUiAction::RetreatRequested,
                InstallerFocusTarget::Forward if self.step == InstallerStep::Summary => {
                    InstallerUiAction::BeginInstallRequested
                }
                InstallerFocusTarget::Forward => InstallerUiAction::AdvanceRequested,
                InstallerFocusTarget::Finish => InstallerUiAction::FinishRequested,
                InstallerFocusTarget::ProgressStatus => InstallerUiAction::None,
            },
        }
    }

    pub fn handle_pointer(
        &mut self,
        layout: &InstallerWindowLayout,
        x: u32,
        y: u32,
    ) -> InstallerUiAction {
        let target = if rect_contains(layout.language_control, x, y) {
            Some(InstallerFocusTarget::LanguageControl)
        } else if self.cancel_visible() && rect_contains(layout.cancel_button, x, y) {
            Some(InstallerFocusTarget::Cancel)
        } else if self.back_visible() && rect_contains(layout.back_button, x, y) {
            Some(InstallerFocusTarget::Back)
        } else if self.forward_label().is_some() && rect_contains(layout.forward_button, x, y) {
            Some(if self.step == InstallerStep::Completed {
                InstallerFocusTarget::Finish
            } else {
                InstallerFocusTarget::Forward
            })
        } else {
            None
        };
        let Some(target) = target else {
            return InstallerUiAction::None;
        };
        self.focus = target;
        match target {
            InstallerFocusTarget::LanguageControl => InstallerUiAction::OpenLanguageControl,
            InstallerFocusTarget::Cancel => InstallerUiAction::CancelRequested,
            InstallerFocusTarget::Back => InstallerUiAction::RetreatRequested,
            InstallerFocusTarget::Forward if self.step == InstallerStep::Summary => {
                InstallerUiAction::BeginInstallRequested
            }
            InstallerFocusTarget::Forward => InstallerUiAction::AdvanceRequested,
            InstallerFocusTarget::Finish => InstallerUiAction::FinishRequested,
            InstallerFocusTarget::StepContent | InstallerFocusTarget::ProgressStatus => {
                InstallerUiAction::None
            }
        }
    }

    pub fn focus_step_content(&mut self) -> InstallerUiAction {
        if !installer_focus_order(self.step).contains(&InstallerFocusTarget::StepContent) {
            return InstallerUiAction::None;
        }
        self.set_focus(InstallerFocusTarget::StepContent)
    }

    pub const fn forward_label(&self) -> Option<&'static str> {
        match self.step {
            InstallerStep::Summary => Some("Kur"),
            InstallerStep::Installation => None,
            InstallerStep::Completed => Some("Yeniden Başlat"),
            _ => Some("İleri"),
        }
    }

    pub const fn back_visible(&self) -> bool {
        !matches!(
            self.step,
            InstallerStep::Welcome | InstallerStep::Installation | InstallerStep::Completed
        )
    }

    pub const fn cancel_visible(&self) -> bool {
        !matches!(
            self.step,
            InstallerStep::Installation | InstallerStep::Completed
        )
    }

    fn move_focus(
        &mut self,
        targets: &'static [InstallerFocusTarget],
        offset: isize,
    ) -> InstallerUiAction {
        let current = targets
            .iter()
            .position(|target| *target == self.focus)
            .unwrap_or(0) as isize;
        let next = (current + offset).rem_euclid(targets.len() as isize) as usize;
        self.set_focus(targets[next])
    }

    fn set_focus(&mut self, focus: InstallerFocusTarget) -> InstallerUiAction {
        self.focus = focus;
        InstallerUiAction::FocusChanged(focus)
    }
}

const fn rect_contains(rect: Rect, x: u32, y: u32) -> bool {
    x >= rect.x && x < rect.right() && y >= rect.y && y < rect.bottom()
}

const WELCOME_FOCUS_ORDER: [InstallerFocusTarget; 3] = [
    InstallerFocusTarget::LanguageControl,
    InstallerFocusTarget::Cancel,
    InstallerFocusTarget::Forward,
];
const FORM_FOCUS_ORDER: [InstallerFocusTarget; 5] = [
    InstallerFocusTarget::StepContent,
    InstallerFocusTarget::LanguageControl,
    InstallerFocusTarget::Cancel,
    InstallerFocusTarget::Back,
    InstallerFocusTarget::Forward,
];
const SUMMARY_FOCUS_ORDER: [InstallerFocusTarget; 4] = [
    InstallerFocusTarget::LanguageControl,
    InstallerFocusTarget::Cancel,
    InstallerFocusTarget::Back,
    InstallerFocusTarget::Forward,
];
const INSTALLATION_FOCUS_ORDER: [InstallerFocusTarget; 1] = [InstallerFocusTarget::ProgressStatus];
const COMPLETED_FOCUS_ORDER: [InstallerFocusTarget; 1] = [InstallerFocusTarget::Finish];

pub const fn installer_focus_order(step: InstallerStep) -> &'static [InstallerFocusTarget] {
    match step {
        InstallerStep::Welcome => &WELCOME_FOCUS_ORDER,
        InstallerStep::Language
        | InstallerStep::Keyboard
        | InstallerStep::Partitions
        | InstallerStep::TimeZone
        | InstallerStep::UserInformation => &FORM_FOCUS_ORDER,
        InstallerStep::Summary => &SUMMARY_FOCUS_ORDER,
        InstallerStep::Installation => &INSTALLATION_FOCUS_ORDER,
        InstallerStep::Completed => &COMPLETED_FOCUS_ORDER,
    }
}

pub const INSTALLER_FORM_STATUS: &str = "validated-language-keyboard-form-controls-ready";
pub const INSTALLER_DISK_FORM_STATUS: &str = "eligible-storage-selection-form-ready";
pub const INSTALLER_TIMEZONE_FORM_STATUS: &str = "validated-timezone-form-control-ready";
pub const INSTALLER_USER_FORM_STATUS: &str = "password-content-free-user-form-ready";
pub const INSTALLER_SUMMARY_FORM_STATUS: &str = "target-bound-summary-confirmation-ready";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallerChoiceOption {
    pub value: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
}

pub const LANGUAGE_OPTIONS: [InstallerChoiceOption; 3] = [
    InstallerChoiceOption {
        value: "tr_TR.UTF-8",
        label: "Türkçe",
        detail: "Türkiye",
    },
    InstallerChoiceOption {
        value: "en_US.UTF-8",
        label: "English",
        detail: "United States",
    },
    InstallerChoiceOption {
        value: "de_DE.UTF-8",
        label: "Deutsch",
        detail: "Deutschland",
    },
];

pub const KEYBOARD_OPTIONS: [InstallerChoiceOption; 3] = [
    InstallerChoiceOption {
        value: "trq",
        label: "Türkçe Q",
        detail: "Standart Türkçe Q düzeni",
    },
    InstallerChoiceOption {
        value: "trf",
        label: "Türkçe F",
        detail: "Standart Türkçe F düzeni",
    },
    InstallerChoiceOption {
        value: "us",
        label: "English (US)",
        detail: "US QWERTY",
    },
];

pub const TIMEZONE_OPTIONS: [InstallerChoiceOption; 4] = [
    InstallerChoiceOption {
        value: "Europe/Istanbul",
        label: "İstanbul",
        detail: "Türkiye",
    },
    InstallerChoiceOption {
        value: "UTC",
        label: "UTC",
        detail: "Eşgüdümlü Evrensel Zaman",
    },
    InstallerChoiceOption {
        value: "Europe/Berlin",
        label: "Berlin",
        detail: "Orta Avrupa",
    },
    InstallerChoiceOption {
        value: "America/New_York",
        label: "New York",
        detail: "Doğu Amerika",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerFormKey {
    Up,
    Down,
    Home,
    End,
    Activate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerFormUpdate {
    None,
    SelectionChanged {
        step: InstallerStep,
        index: usize,
        value: &'static str,
    },
    ValueApplied {
        step: InstallerStep,
        index: usize,
        value: &'static str,
    },
}

impl InstallerFormUpdate {
    pub const fn changed(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerDiskOption {
    candidate: StorageCandidate,
}

impl InstallerDiskOption {
    fn from_candidate(candidate: StorageCandidate) -> Self {
        Self { candidate }
    }

    pub fn device(&self) -> &str {
        self.candidate.device()
    }

    pub fn model(&self) -> &str {
        self.candidate.model()
    }

    pub const fn capacity_bytes(&self) -> u64 {
        self.candidate.capacity_bytes()
    }

    pub const fn removable(&self) -> bool {
        self.candidate.removable()
    }

    pub fn blocked_reasons(&self) -> &[StorageBlockReason] {
        self.candidate.blocked_reasons()
    }

    pub fn is_eligible(&self) -> bool {
        self.candidate.is_eligible()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerDiskFormUpdate {
    None,
    SelectionChanged { index: usize },
    TargetApplied { index: usize },
}

impl InstallerDiskFormUpdate {
    pub const fn changed(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InstallerUserField {
    #[default]
    Username,
    DisplayName,
    Password,
}

impl InstallerUserField {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Username => "username",
            Self::DisplayName => "display-name",
            Self::Password => "password",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerUserFormKey {
    Character(char),
    Backspace,
    NextField,
    PreviousField,
    SetPasswordConfigured(bool),
    Activate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerUserFormUpdate {
    None,
    FieldChanged(InstallerUserField),
    TextChanged(InstallerUserField),
    PasswordStatusChanged(bool),
    ProfileApplied,
}

impl InstallerUserFormUpdate {
    pub const fn changed(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallerUserFormState {
    username: String,
    display_name: String,
    active_field: InstallerUserField,
    password_configured: bool,
}

impl InstallerUserFormState {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub const fn active_field(&self) -> InstallerUserField {
        self.active_field
    }

    pub const fn password_configured(&self) -> bool {
        self.password_configured
    }

    pub fn sync_model(&mut self, model: &InstallerModel) {
        if let Some(user) = model.user() {
            self.username = user.username().to_string();
            self.display_name = user.display_name().to_string();
            self.password_configured = user.password_configured();
        }
    }

    pub fn handle_key(
        &mut self,
        model: &mut InstallerModel,
        key: InstallerUserFormKey,
    ) -> Result<InstallerUserFormUpdate, InstallerError> {
        if model.step() != InstallerStep::UserInformation {
            return Ok(InstallerUserFormUpdate::None);
        }
        match key {
            InstallerUserFormKey::NextField => {
                self.active_field = match self.active_field {
                    InstallerUserField::Username => InstallerUserField::DisplayName,
                    InstallerUserField::DisplayName => InstallerUserField::Password,
                    InstallerUserField::Password => InstallerUserField::Username,
                };
                Ok(InstallerUserFormUpdate::FieldChanged(self.active_field))
            }
            InstallerUserFormKey::PreviousField => {
                self.active_field = match self.active_field {
                    InstallerUserField::Username => InstallerUserField::Password,
                    InstallerUserField::DisplayName => InstallerUserField::Username,
                    InstallerUserField::Password => InstallerUserField::DisplayName,
                };
                Ok(InstallerUserFormUpdate::FieldChanged(self.active_field))
            }
            InstallerUserFormKey::Character(character) => {
                let changed = match self.active_field {
                    InstallerUserField::Username
                        if self.username.len() < 32
                            && (character.is_ascii_lowercase()
                                || character.is_ascii_digit()
                                || character == '_') =>
                    {
                        self.username.push(character);
                        true
                    }
                    InstallerUserField::DisplayName
                        if !character.is_control()
                            && self.display_name.len() + character.len_utf8() <= 128 =>
                    {
                        self.display_name.push(character);
                        true
                    }
                    InstallerUserField::Username
                    | InstallerUserField::DisplayName
                    | InstallerUserField::Password => false,
                };
                Ok(if changed {
                    InstallerUserFormUpdate::TextChanged(self.active_field)
                } else {
                    InstallerUserFormUpdate::None
                })
            }
            InstallerUserFormKey::Backspace => {
                let changed = match self.active_field {
                    InstallerUserField::Username => self.username.pop().is_some(),
                    InstallerUserField::DisplayName => self.display_name.pop().is_some(),
                    InstallerUserField::Password => false,
                };
                Ok(if changed {
                    InstallerUserFormUpdate::TextChanged(self.active_field)
                } else {
                    InstallerUserFormUpdate::None
                })
            }
            InstallerUserFormKey::SetPasswordConfigured(configured) => {
                self.password_configured = configured;
                Ok(InstallerUserFormUpdate::PasswordStatusChanged(configured))
            }
            InstallerUserFormKey::Activate => {
                let profile = UserProfile::new(
                    self.username.clone(),
                    self.display_name.clone(),
                    self.password_configured,
                )?;
                model.set_user(profile);
                Ok(InstallerUserFormUpdate::ProfileApplied)
            }
        }
    }

    pub fn handle_pointer(
        &mut self,
        model: &InstallerModel,
        layout: &InstallerWindowLayout,
        x: u32,
        y: u32,
    ) -> InstallerUserFormUpdate {
        if model.step() != InstallerStep::UserInformation {
            return InstallerUserFormUpdate::None;
        }
        for field in [
            InstallerUserField::Username,
            InstallerUserField::DisplayName,
            InstallerUserField::Password,
        ] {
            if rect_contains(layout.user_field_row(field), x, y) {
                self.active_field = field;
                return InstallerUserFormUpdate::FieldChanged(field);
            }
        }
        InstallerUserFormUpdate::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerSummaryKey {
    Character(char),
    Backspace,
    Clear,
    Activate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerSummaryUpdate {
    None,
    ConfirmationChanged,
    ConfirmationApplied,
    ReadyToInstall,
}

impl InstallerSummaryUpdate {
    pub const fn changed(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallerSummaryState {
    confirmation: String,
}

impl InstallerSummaryState {
    pub fn confirmation(&self) -> &str {
        &self.confirmation
    }

    pub fn can_begin_install(&self, model: &InstallerModel) -> bool {
        model.step() == InstallerStep::Summary
            && (model.mode() == InstallMode::DryRun || model.destructive_confirmed())
    }

    pub fn handle_key(
        &mut self,
        model: &mut InstallerModel,
        key: InstallerSummaryKey,
    ) -> Result<InstallerSummaryUpdate, InstallerError> {
        if model.step() != InstallerStep::Summary {
            return Ok(InstallerSummaryUpdate::None);
        }
        match key {
            InstallerSummaryKey::Character(character)
                if !character.is_control()
                    && self.confirmation.len() + character.len_utf8() <= 160 =>
            {
                self.confirmation.push(character);
                Ok(InstallerSummaryUpdate::ConfirmationChanged)
            }
            InstallerSummaryKey::Character(_) => Ok(InstallerSummaryUpdate::None),
            InstallerSummaryKey::Backspace => Ok(if self.confirmation.pop().is_some() {
                InstallerSummaryUpdate::ConfirmationChanged
            } else {
                InstallerSummaryUpdate::None
            }),
            InstallerSummaryKey::Clear => {
                if self.confirmation.is_empty() {
                    Ok(InstallerSummaryUpdate::None)
                } else {
                    self.confirmation.clear();
                    Ok(InstallerSummaryUpdate::ConfirmationChanged)
                }
            }
            InstallerSummaryKey::Activate if model.mode() == InstallMode::DryRun => {
                Ok(InstallerSummaryUpdate::ReadyToInstall)
            }
            InstallerSummaryKey::Activate => {
                model.confirm_destructive(&self.confirmation)?;
                Ok(InstallerSummaryUpdate::ConfirmationApplied)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallerFormState {
    language_index: usize,
    keyboard_index: usize,
    timezone_index: usize,
    disk_options: Vec<InstallerDiskOption>,
    disk_index: Option<usize>,
    user: InstallerUserFormState,
    summary: InstallerSummaryState,
}

impl InstallerFormState {
    pub const fn language_index(&self) -> usize {
        self.language_index
    }

    pub const fn keyboard_index(&self) -> usize {
        self.keyboard_index
    }

    pub const fn timezone_index(&self) -> usize {
        self.timezone_index
    }

    pub fn disk_options(&self) -> &[InstallerDiskOption] {
        &self.disk_options
    }

    pub const fn disk_index(&self) -> Option<usize> {
        self.disk_index
    }

    pub const fn user(&self) -> &InstallerUserFormState {
        &self.user
    }

    pub fn user_mut(&mut self) -> &mut InstallerUserFormState {
        &mut self.user
    }

    pub const fn summary(&self) -> &InstallerSummaryState {
        &self.summary
    }

    pub fn summary_mut(&mut self) -> &mut InstallerSummaryState {
        &mut self.summary
    }

    pub fn load_storage_inventory(&mut self, inventory: &StorageInventory) {
        self.disk_options = inventory
            .candidates()
            .iter()
            .cloned()
            .map(InstallerDiskOption::from_candidate)
            .collect();
        self.disk_index = self
            .disk_options
            .iter()
            .position(InstallerDiskOption::is_eligible);
    }

    pub fn load_selected_target(&mut self, target: &InstallTarget) {
        self.disk_options = vec![InstallerDiskOption::from_candidate(StorageCandidate {
            device: target.disk.device().to_string(),
            stable_id: target.disk.stable_id().to_string(),
            model: target.disk.model().to_string(),
            capacity_bytes: target.disk.capacity_bytes(),
            removable: false,
            blocked_reasons: Vec::new(),
        })];
        self.disk_index = Some(0);
    }

    pub fn selected_index(&self, step: InstallerStep) -> Option<usize> {
        match step {
            InstallerStep::Language => Some(self.language_index),
            InstallerStep::Keyboard => Some(self.keyboard_index),
            InstallerStep::TimeZone => Some(self.timezone_index),
            _ => None,
        }
    }

    pub fn sync_model(&mut self, model: &InstallerModel) {
        if let Some(locale) = model.locale() {
            if let Some(index) = LANGUAGE_OPTIONS
                .iter()
                .position(|option| option.value == locale)
            {
                self.language_index = index;
            }
        }
        if let Some(layout) = model.keyboard_layout() {
            if let Some(index) = KEYBOARD_OPTIONS
                .iter()
                .position(|option| option.value == layout)
            {
                self.keyboard_index = index;
            }
        }
        if let Some(target) = model.target() {
            self.disk_index = self
                .disk_options
                .iter()
                .position(|option| option.device() == target.disk.device());
        }
        if let Some(timezone) = model.timezone() {
            if let Some(index) = TIMEZONE_OPTIONS
                .iter()
                .position(|option| option.value == timezone)
            {
                self.timezone_index = index;
            }
        }
        self.user.sync_model(model);
    }

    pub fn handle_key(
        &mut self,
        model: &mut InstallerModel,
        key: InstallerFormKey,
    ) -> Result<InstallerFormUpdate, InstallerError> {
        let (index, options) = match model.step() {
            InstallerStep::Language => (&mut self.language_index, &LANGUAGE_OPTIONS[..]),
            InstallerStep::Keyboard => (&mut self.keyboard_index, &KEYBOARD_OPTIONS[..]),
            InstallerStep::TimeZone => (&mut self.timezone_index, &TIMEZONE_OPTIONS[..]),
            _ => return Ok(InstallerFormUpdate::None),
        };
        let previous = *index;
        *index = match key {
            InstallerFormKey::Up => index.checked_sub(1).unwrap_or(options.len() - 1),
            InstallerFormKey::Down => (*index + 1) % options.len(),
            InstallerFormKey::Home => 0,
            InstallerFormKey::End => options.len() - 1,
            InstallerFormKey::Activate => *index,
        };
        let selected = options[*index];
        match model.step() {
            InstallerStep::Language => model.set_locale(selected.value)?,
            InstallerStep::Keyboard => model.set_keyboard_layout(selected.value)?,
            InstallerStep::TimeZone => model.set_timezone(selected.value)?,
            _ => unreachable!("form step checked above"),
        }
        Ok(
            if matches!(key, InstallerFormKey::Activate) || previous == *index {
                InstallerFormUpdate::ValueApplied {
                    step: model.step(),
                    index: *index,
                    value: selected.value,
                }
            } else {
                InstallerFormUpdate::SelectionChanged {
                    step: model.step(),
                    index: *index,
                    value: selected.value,
                }
            },
        )
    }

    pub fn handle_choice_pointer(
        &mut self,
        model: &mut InstallerModel,
        layout: &InstallerWindowLayout,
        x: u32,
        y: u32,
    ) -> Result<InstallerFormUpdate, InstallerError> {
        let (index, options) = match model.step() {
            InstallerStep::Language => (&mut self.language_index, &LANGUAGE_OPTIONS[..]),
            InstallerStep::Keyboard => (&mut self.keyboard_index, &KEYBOARD_OPTIONS[..]),
            InstallerStep::TimeZone => (&mut self.timezone_index, &TIMEZONE_OPTIONS[..]),
            _ => return Ok(InstallerFormUpdate::None),
        };
        let Some(selected_index) = options.iter().enumerate().find_map(|(candidate, _)| {
            rect_contains(layout.choice_row(candidate), x, y).then_some(candidate)
        }) else {
            return Ok(InstallerFormUpdate::None);
        };
        *index = selected_index;
        let selected = options[selected_index];
        match model.step() {
            InstallerStep::Language => model.set_locale(selected.value)?,
            InstallerStep::Keyboard => model.set_keyboard_layout(selected.value)?,
            InstallerStep::TimeZone => model.set_timezone(selected.value)?,
            _ => unreachable!("form step checked above"),
        }
        Ok(InstallerFormUpdate::SelectionChanged {
            step: model.step(),
            index: selected_index,
            value: selected.value,
        })
    }

    pub fn handle_disk_key(
        &mut self,
        model: &mut InstallerModel,
        key: InstallerFormKey,
    ) -> Result<InstallerDiskFormUpdate, StorageProbeError> {
        if model.step() != InstallerStep::Partitions {
            return Ok(InstallerDiskFormUpdate::None);
        }
        let eligible = self
            .disk_options
            .iter()
            .enumerate()
            .filter_map(|(index, option)| option.is_eligible().then_some(index))
            .collect::<Vec<_>>();
        if eligible.is_empty() {
            self.disk_index = None;
            return Ok(InstallerDiskFormUpdate::None);
        }
        let current_position = self
            .disk_index
            .and_then(|index| eligible.iter().position(|candidate| *candidate == index))
            .unwrap_or(0);
        let next_position = match key {
            InstallerFormKey::Up => current_position
                .checked_sub(1)
                .unwrap_or(eligible.len() - 1),
            InstallerFormKey::Down => (current_position + 1) % eligible.len(),
            InstallerFormKey::Home => 0,
            InstallerFormKey::End => eligible.len() - 1,
            InstallerFormKey::Activate => current_position,
        };
        let selected_index = eligible[next_position];
        self.disk_index = Some(selected_index);
        if matches!(key, InstallerFormKey::Activate) {
            let target = self.disk_options[selected_index]
                .candidate
                .clone()
                .into_erase_target()?;
            model.set_target(target);
            Ok(InstallerDiskFormUpdate::TargetApplied {
                index: selected_index,
            })
        } else if next_position == current_position {
            Ok(InstallerDiskFormUpdate::None)
        } else {
            Ok(InstallerDiskFormUpdate::SelectionChanged {
                index: selected_index,
            })
        }
    }

    pub fn handle_disk_pointer(
        &mut self,
        model: &InstallerModel,
        layout: &InstallerWindowLayout,
        x: u32,
        y: u32,
    ) -> InstallerDiskFormUpdate {
        if model.step() != InstallerStep::Partitions {
            return InstallerDiskFormUpdate::None;
        }
        let Some(index) =
            self.disk_options
                .iter()
                .take(4)
                .enumerate()
                .find_map(|(index, option)| {
                    (option.is_eligible() && rect_contains(layout.disk_row(index), x, y))
                        .then_some(index)
                })
        else {
            return InstallerDiskFormUpdate::None;
        };
        self.disk_index = Some(index);
        InstallerDiskFormUpdate::SelectionChanged { index }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InstallMode {
    #[default]
    DryRun,
    Real,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filesystem {
    Ext4,
}

impl Filesystem {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Ext4 => "ext4",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskIdentity {
    device: String,
    stable_id: String,
    model: String,
    capacity_bytes: u64,
}

impl DiskIdentity {
    pub fn new(
        device: impl Into<String>,
        stable_id: impl Into<String>,
        model: impl Into<String>,
        capacity_bytes: u64,
    ) -> Result<Self, InstallerError> {
        let device = device.into();
        let stable_id = stable_id.into();
        let model = model.into();
        if !device.starts_with("/dev/") || device.len() > 128 || contains_control(&device) {
            return Err(InstallerError::InvalidDiskIdentity);
        }
        if stable_id.trim().is_empty()
            || stable_id.len() > 256
            || contains_control(&stable_id)
            || model.trim().is_empty()
            || model.len() > 128
            || contains_control(&model)
            || capacity_bytes == 0
        {
            return Err(InstallerError::InvalidDiskIdentity);
        }
        Ok(Self {
            device,
            stable_id,
            model,
            capacity_bytes,
        })
    }

    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn stable_id(&self) -> &str {
        &self.stable_id
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub const fn capacity_bytes(&self) -> u64 {
        self.capacity_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTarget {
    pub disk: DiskIdentity,
    pub filesystem: Filesystem,
    pub erase_disk: bool,
}

impl InstallTarget {
    pub fn erase_disk(disk: DiskIdentity) -> Self {
        Self {
            disk,
            filesystem: Filesystem::Ext4,
            erase_disk: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    username: String,
    display_name: String,
    password_configured: bool,
}

impl UserProfile {
    pub fn new(
        username: impl Into<String>,
        display_name: impl Into<String>,
        password_configured: bool,
    ) -> Result<Self, InstallerError> {
        let username = username.into();
        let display_name = display_name.into();
        let username_valid = (1..=32).contains(&username.len())
            && username
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase())
            && username
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
        if !username_valid
            || display_name.trim().is_empty()
            || display_name.len() > 128
            || contains_control(&display_name)
            || !password_configured
        {
            return Err(InstallerError::InvalidUserProfile);
        }
        Ok(Self {
            username,
            display_name,
            password_configured,
        })
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub const fn password_configured(&self) -> bool {
        self.password_configured
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallerError {
    MissingSelection(InstallerStep),
    InvalidDiskIdentity,
    InvalidUserProfile,
    InvalidValue(&'static str),
    CannotMoveBack(InstallerStep),
    BeginInstallRequired,
    NotAtSummary,
    DestructiveConfirmationRequired,
    ConfirmationPhraseMismatch,
    InstallationNotRunning,
}

impl fmt::Display for InstallerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSelection(step) => {
                write!(formatter, "missing selection for {}", step.id())
            }
            Self::InvalidDiskIdentity => formatter.write_str("invalid disk identity"),
            Self::InvalidUserProfile => formatter.write_str("invalid user profile"),
            Self::InvalidValue(field) => write!(formatter, "invalid value for {field}"),
            Self::CannotMoveBack(step) => write!(formatter, "cannot move back from {}", step.id()),
            Self::BeginInstallRequired => formatter.write_str("use begin_install at summary"),
            Self::NotAtSummary => formatter.write_str("installer is not at summary"),
            Self::DestructiveConfirmationRequired => {
                formatter.write_str("destructive confirmation required")
            }
            Self::ConfirmationPhraseMismatch => {
                formatter.write_str("destructive confirmation phrase mismatch")
            }
            Self::InstallationNotRunning => formatter.write_str("installation is not running"),
        }
    }
}

impl Error for InstallerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerModel {
    step: InstallerStep,
    mode: InstallMode,
    locale: Option<String>,
    keyboard_layout: Option<String>,
    target: Option<InstallTarget>,
    timezone: Option<String>,
    user: Option<UserProfile>,
    destructive_confirmed: bool,
}

impl Default for InstallerModel {
    fn default() -> Self {
        Self {
            step: InstallerStep::Welcome,
            mode: InstallMode::DryRun,
            locale: None,
            keyboard_layout: None,
            target: None,
            timezone: None,
            user: None,
            destructive_confirmed: false,
        }
    }
}

impl InstallerModel {
    pub fn step(&self) -> InstallerStep {
        self.step
    }

    pub fn mode(&self) -> InstallMode {
        self.mode
    }

    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    pub fn keyboard_layout(&self) -> Option<&str> {
        self.keyboard_layout.as_deref()
    }

    pub fn target(&self) -> Option<&InstallTarget> {
        self.target.as_ref()
    }

    pub fn timezone(&self) -> Option<&str> {
        self.timezone.as_deref()
    }

    pub fn user(&self) -> Option<&UserProfile> {
        self.user.as_ref()
    }

    pub const fn destructive_confirmed(&self) -> bool {
        self.destructive_confirmed
    }

    pub fn set_mode(&mut self, mode: InstallMode) {
        if self.mode != mode {
            self.mode = mode;
            self.destructive_confirmed = false;
        }
    }

    pub fn set_locale(&mut self, locale: impl Into<String>) -> Result<(), InstallerError> {
        self.locale = Some(validate_bounded_value("locale", locale.into(), 64)?);
        Ok(())
    }

    pub fn set_keyboard_layout(
        &mut self,
        keyboard_layout: impl Into<String>,
    ) -> Result<(), InstallerError> {
        self.keyboard_layout = Some(validate_bounded_value(
            "keyboard-layout",
            keyboard_layout.into(),
            64,
        )?);
        Ok(())
    }

    pub fn set_target(&mut self, target: InstallTarget) {
        if self.target.as_ref() != Some(&target) {
            self.target = Some(target);
            self.destructive_confirmed = false;
        }
    }

    pub fn set_timezone(&mut self, timezone: impl Into<String>) -> Result<(), InstallerError> {
        self.timezone = Some(validate_bounded_value("timezone", timezone.into(), 128)?);
        Ok(())
    }

    pub fn set_user(&mut self, user: UserProfile) {
        self.user = Some(user);
    }

    pub fn advance(&mut self) -> Result<InstallerStep, InstallerError> {
        self.validate_current_step()?;
        let next = self.step.next().ok_or(match self.step {
            InstallerStep::Summary => InstallerError::BeginInstallRequired,
            _ => InstallerError::MissingSelection(self.step),
        })?;
        self.step = next;
        Ok(next)
    }

    pub fn retreat(&mut self) -> Result<InstallerStep, InstallerError> {
        let previous = self
            .step
            .previous()
            .ok_or(InstallerError::CannotMoveBack(self.step))?;
        self.step = previous;
        self.destructive_confirmed = false;
        Ok(previous)
    }

    pub fn confirmation_phrase(&self) -> Option<String> {
        (self.mode == InstallMode::Real).then(|| {
            self.target
                .as_ref()
                .map(|target| format!("ERASE {}", target.disk.device()))
        })?
    }

    pub fn confirm_destructive(&mut self, phrase: &str) -> Result<(), InstallerError> {
        if self.step != InstallerStep::Summary {
            return Err(InstallerError::NotAtSummary);
        }
        let expected = self
            .confirmation_phrase()
            .ok_or(InstallerError::DestructiveConfirmationRequired)?;
        if phrase != expected {
            self.destructive_confirmed = false;
            return Err(InstallerError::ConfirmationPhraseMismatch);
        }
        self.destructive_confirmed = true;
        Ok(())
    }

    pub fn begin_install(&mut self) -> Result<(), InstallerError> {
        if self.step != InstallerStep::Summary {
            return Err(InstallerError::NotAtSummary);
        }
        self.validate_all_selections()?;
        if self.mode == InstallMode::Real && !self.destructive_confirmed {
            return Err(InstallerError::DestructiveConfirmationRequired);
        }
        self.step = InstallerStep::Installation;
        Ok(())
    }

    pub fn complete_install(&mut self) -> Result<(), InstallerError> {
        if self.step != InstallerStep::Installation {
            return Err(InstallerError::InstallationNotRunning);
        }
        self.step = InstallerStep::Completed;
        Ok(())
    }

    fn validate_current_step(&self) -> Result<(), InstallerError> {
        let selected = match self.step {
            InstallerStep::Welcome => true,
            InstallerStep::Language => self.locale.is_some(),
            InstallerStep::Keyboard => self.keyboard_layout.is_some(),
            InstallerStep::Partitions => self.target.is_some(),
            InstallerStep::TimeZone => self.timezone.is_some(),
            InstallerStep::UserInformation => self.user.is_some(),
            InstallerStep::Summary => return Err(InstallerError::BeginInstallRequired),
            InstallerStep::Installation | InstallerStep::Completed => false,
        };
        selected
            .then_some(())
            .ok_or(InstallerError::MissingSelection(self.step))
    }

    fn validate_all_selections(&self) -> Result<(), InstallerError> {
        for (step, selected) in [
            (InstallerStep::Language, self.locale.is_some()),
            (InstallerStep::Keyboard, self.keyboard_layout.is_some()),
            (InstallerStep::Partitions, self.target.is_some()),
            (InstallerStep::TimeZone, self.timezone.is_some()),
            (InstallerStep::UserInformation, self.user.is_some()),
        ] {
            if !selected {
                return Err(InstallerError::MissingSelection(step));
            }
        }
        Ok(())
    }
}

pub const DRY_RUN_PLAN_STATUS: &str = "deterministic-dry-run-plan-ready";
pub const INSTALL_PLAN_VERSION: u8 = 1;
pub const INSTALL_MINIMUM_CAPACITY_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const INSTALL_ESP_START_MIB: u64 = 1;
pub const INSTALL_ESP_SIZE_MIB: u64 = 512;
pub const INSTALL_ESP_LABEL: &str = "AQUA_EFI";
pub const INSTALL_ROOT_LABEL: &str = "AQUA_ROOT";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootloaderStrategy {
    Grub2X86_64Efi,
}

impl BootloaderStrategy {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Grub2X86_64Efi => "grub2-x86_64-efi",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallArtifacts {
    rootfs_archive: PathBuf,
    kernel_image: PathBuf,
    bootloader_image: PathBuf,
}

impl InstallArtifacts {
    pub fn new(
        rootfs_archive: impl Into<PathBuf>,
        kernel_image: impl Into<PathBuf>,
        bootloader_image: impl Into<PathBuf>,
    ) -> Result<Self, InstallPlanError> {
        let rootfs_archive = rootfs_archive.into();
        let kernel_image = kernel_image.into();
        let bootloader_image = bootloader_image.into();
        validate_plan_path(&rootfs_archive, "rootfs-archive")?;
        validate_plan_path(&kernel_image, "kernel-image")?;
        validate_plan_path(&bootloader_image, "bootloader-image")?;
        Ok(Self {
            rootfs_archive,
            kernel_image,
            bootloader_image,
        })
    }

    pub fn rootfs_archive(&self) -> &Path {
        &self.rootfs_archive
    }

    pub fn kernel_image(&self) -> &Path {
        &self.kernel_image
    }

    pub fn bootloader_image(&self) -> &Path {
        &self.bootloader_image
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallPlanBlocker {}

impl InstallPlanBlocker {
    pub const fn id(self) -> &'static str {
        match self {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPlanOperation {
    VerifyTarget {
        device: String,
        stable_id: String,
        model: String,
        capacity_bytes: u64,
    },
    WriteGpt {
        device: String,
    },
    CreateEfiSystemPartition {
        partition: String,
        start_mib: u64,
        size_mib: u64,
    },
    CreateRootPartition {
        partition: String,
        start_mib: u64,
        filesystem: Filesystem,
    },
    FormatEfiSystemPartition {
        partition: String,
        label: &'static str,
    },
    FormatRootPartition {
        partition: String,
        filesystem: Filesystem,
        label: &'static str,
    },
    MountRoot {
        partition: String,
        mountpoint: &'static str,
    },
    ExtractRootFilesystem {
        source: PathBuf,
        destination: &'static str,
    },
    MountEfiSystemPartition {
        partition: String,
        mountpoint: &'static str,
    },
    InstallKernel {
        source: PathBuf,
        destination: &'static str,
    },
    InstallBootloader {
        strategy: BootloaderStrategy,
        source: PathBuf,
        destination: &'static str,
        config_destination: &'static str,
        root_label: &'static str,
        kernel_path: &'static str,
        kernel_cmdline: &'static str,
    },
    WriteSystemConfiguration {
        locale: String,
        keyboard_layout: String,
        timezone: String,
        username: String,
        display_name: String,
        password_configured: bool,
    },
    UnmountTarget {
        mountpoint: &'static str,
    },
}

impl InstallPlanOperation {
    pub const fn id(&self) -> &'static str {
        match self {
            Self::VerifyTarget { .. } => "verify-target",
            Self::WriteGpt { .. } => "write-gpt",
            Self::CreateEfiSystemPartition { .. } => "create-efi-system-partition",
            Self::CreateRootPartition { .. } => "create-root-partition",
            Self::FormatEfiSystemPartition { .. } => "format-efi-system-partition",
            Self::FormatRootPartition { .. } => "format-root-partition",
            Self::MountRoot { .. } => "mount-root",
            Self::ExtractRootFilesystem { .. } => "extract-root-filesystem",
            Self::MountEfiSystemPartition { .. } => "mount-efi-system-partition",
            Self::InstallKernel { .. } => "install-kernel",
            Self::InstallBootloader { .. } => "install-bootloader",
            Self::WriteSystemConfiguration { .. } => "write-system-configuration",
            Self::UnmountTarget { .. } => "unmount-target",
        }
    }

    fn render(&self) -> String {
        match self {
            Self::VerifyTarget {
                device,
                stable_id,
                model,
                capacity_bytes,
            } => format!(
                "verify-target device={device} stable_id={stable_id} model={} capacity_bytes={capacity_bytes}",
                encode_plan_value(model)
            ),
            Self::WriteGpt { device } => format!("write-gpt device={device}"),
            Self::CreateEfiSystemPartition {
                partition,
                start_mib,
                size_mib,
            } => format!(
                "create-efi-system-partition partition={partition} start_mib={start_mib} size_mib={size_mib} type=esp"
            ),
            Self::CreateRootPartition {
                partition,
                start_mib,
                filesystem,
            } => format!(
                "create-root-partition partition={partition} start_mib={start_mib} end=100% filesystem={}",
                filesystem.id()
            ),
            Self::FormatEfiSystemPartition { partition, label } => {
                format!("format-efi-system-partition partition={partition} filesystem=fat32 label={label}")
            }
            Self::FormatRootPartition {
                partition,
                filesystem,
                label,
            } => format!(
                "format-root-partition partition={partition} filesystem={} label={label}",
                filesystem.id(),
            ),
            Self::MountRoot {
                partition,
                mountpoint,
            } => format!("mount-root partition={partition} mountpoint={mountpoint}"),
            Self::ExtractRootFilesystem {
                source,
                destination,
            } => format!(
                "extract-root-filesystem source={} destination={destination}",
                source.display()
            ),
            Self::MountEfiSystemPartition {
                partition,
                mountpoint,
            } => format!(
                "mount-efi-system-partition partition={partition} mountpoint={mountpoint}"
            ),
            Self::InstallKernel {
                source,
                destination,
            } => format!(
                "install-kernel source={} destination={destination}",
                source.display()
            ),
            Self::InstallBootloader {
                strategy,
                source,
                destination,
                config_destination,
                root_label,
                kernel_path,
                kernel_cmdline,
            } => format!(
                "install-bootloader strategy={} source={} destination={destination} config_destination={config_destination} root_label={root_label} kernel_path={kernel_path} kernel_cmdline={}",
                strategy.id(),
                source.display(),
                encode_plan_value(kernel_cmdline)
            ),
            Self::WriteSystemConfiguration {
                locale,
                keyboard_layout,
                timezone,
                username,
                display_name,
                password_configured,
            } => format!(
                "write-system-configuration locale={} keyboard={} timezone={} username={} display_name={} password_configured={password_configured}",
                encode_plan_value(locale),
                encode_plan_value(keyboard_layout),
                encode_plan_value(timezone),
                encode_plan_value(username),
                encode_plan_value(display_name)
            ),
            Self::UnmountTarget { mountpoint } => {
                format!("unmount-target mountpoint={mountpoint}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    version: u8,
    source_mode: InstallMode,
    target_device: String,
    operations: Vec<InstallPlanOperation>,
    blockers: Vec<InstallPlanBlocker>,
    fingerprint: u64,
}

impl InstallPlan {
    pub const fn version(&self) -> u8 {
        self.version
    }

    pub const fn source_mode(&self) -> InstallMode {
        self.source_mode
    }

    pub fn target_device(&self) -> &str {
        &self.target_device
    }

    pub fn operations(&self) -> &[InstallPlanOperation] {
        &self.operations
    }

    pub fn blockers(&self) -> &[InstallPlanBlocker] {
        &self.blockers
    }

    pub const fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    pub const fn execution_allowed(&self) -> bool {
        false
    }

    pub fn render(&self) -> String {
        let mut lines = vec![
            format!("aqua_install_plan_version={}", self.version),
            "plan_mode=dry-run".to_string(),
            format!(
                "source_mode={}",
                match self.source_mode {
                    InstallMode::DryRun => "dry-run",
                    InstallMode::Real => "real",
                }
            ),
            "execution_allowed=false".to_string(),
            format!("target_device={}", self.target_device),
            format!("operation_count={}", self.operations.len()),
        ];
        lines.extend(
            self.operations
                .iter()
                .enumerate()
                .map(|(index, operation)| {
                    format!("operation.{:02}={}", index + 1, operation.render())
                }),
        );
        lines.push(format!("blocker_count={}", self.blockers.len()));
        lines.extend(
            self.blockers
                .iter()
                .enumerate()
                .map(|(index, blocker)| format!("blocker.{:02}={}", index + 1, blocker.id())),
        );
        lines.push(format!("plan_fingerprint={:016x}", self.fingerprint));
        lines.push("[AQUA-INSTALLER] stage=dry-run-plan status=ok executed=false".to_string());
        lines.join("\n") + "\n"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPlanError {
    Installer(InstallerError),
    NotAtSummary,
    InvalidArtifactPath(&'static str),
    InsufficientCapacity {
        available_bytes: u64,
        required_bytes: u64,
    },
}

impl fmt::Display for InstallPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Installer(error) => write!(formatter, "installer state is incomplete: {error}"),
            Self::NotAtSummary => formatter.write_str("dry-run plan requires summary step"),
            Self::InvalidArtifactPath(field) => write!(formatter, "invalid {field} path"),
            Self::InsufficientCapacity {
                available_bytes,
                required_bytes,
            } => write!(
                formatter,
                "target capacity {available_bytes} is below required {required_bytes}"
            ),
        }
    }
}

impl Error for InstallPlanError {}

pub fn build_dry_run_plan(
    model: &InstallerModel,
    artifacts: &InstallArtifacts,
) -> Result<InstallPlan, InstallPlanError> {
    if model.step != InstallerStep::Summary {
        return Err(InstallPlanError::NotAtSummary);
    }
    model
        .validate_all_selections()
        .map_err(InstallPlanError::Installer)?;
    let target = model.target.as_ref().expect("validated target selection");
    if target.disk.capacity_bytes() < INSTALL_MINIMUM_CAPACITY_BYTES {
        return Err(InstallPlanError::InsufficientCapacity {
            available_bytes: target.disk.capacity_bytes(),
            required_bytes: INSTALL_MINIMUM_CAPACITY_BYTES,
        });
    }
    let locale = model.locale.as_ref().expect("validated locale selection");
    let keyboard = model
        .keyboard_layout
        .as_ref()
        .expect("validated keyboard selection");
    let timezone = model
        .timezone
        .as_ref()
        .expect("validated timezone selection");
    let user = model.user.as_ref().expect("validated user selection");
    let device = target.disk.device().to_string();
    let efi_partition = partition_path(&device, 1);
    let root_partition = partition_path(&device, 2);
    let operations = vec![
        InstallPlanOperation::VerifyTarget {
            device: device.clone(),
            stable_id: target.disk.stable_id().to_string(),
            model: target.disk.model().to_string(),
            capacity_bytes: target.disk.capacity_bytes(),
        },
        InstallPlanOperation::WriteGpt {
            device: device.clone(),
        },
        InstallPlanOperation::CreateEfiSystemPartition {
            partition: efi_partition.clone(),
            start_mib: INSTALL_ESP_START_MIB,
            size_mib: INSTALL_ESP_SIZE_MIB,
        },
        InstallPlanOperation::CreateRootPartition {
            partition: root_partition.clone(),
            start_mib: INSTALL_ESP_START_MIB + INSTALL_ESP_SIZE_MIB,
            filesystem: target.filesystem,
        },
        InstallPlanOperation::FormatEfiSystemPartition {
            partition: efi_partition.clone(),
            label: INSTALL_ESP_LABEL,
        },
        InstallPlanOperation::FormatRootPartition {
            partition: root_partition.clone(),
            filesystem: target.filesystem,
            label: INSTALL_ROOT_LABEL,
        },
        InstallPlanOperation::MountRoot {
            partition: root_partition,
            mountpoint: "/mnt/aqua-target",
        },
        InstallPlanOperation::ExtractRootFilesystem {
            source: artifacts.rootfs_archive.clone(),
            destination: "/mnt/aqua-target",
        },
        InstallPlanOperation::MountEfiSystemPartition {
            partition: efi_partition,
            mountpoint: "/mnt/aqua-target/boot/efi",
        },
        InstallPlanOperation::InstallKernel {
            source: artifacts.kernel_image.clone(),
            destination: "/mnt/aqua-target/boot/vmlinuz-aqua",
        },
        InstallPlanOperation::InstallBootloader {
            strategy: BootloaderStrategy::Grub2X86_64Efi,
            source: artifacts.bootloader_image.clone(),
            destination: "/mnt/aqua-target/boot/efi/EFI/BOOT/BOOTX64.EFI",
            config_destination: "/mnt/aqua-target/boot/efi/EFI/BOOT/grub.cfg",
            root_label: INSTALL_ROOT_LABEL,
            kernel_path: "/boot/vmlinuz-aqua",
            kernel_cmdline:
                "root=PARTLABEL=AQUA_ROOT rootwait rw console=tty0 console=ttyS0,115200",
        },
        InstallPlanOperation::WriteSystemConfiguration {
            locale: locale.clone(),
            keyboard_layout: keyboard.clone(),
            timezone: timezone.clone(),
            username: user.username().to_string(),
            display_name: user.display_name().to_string(),
            password_configured: user.password_configured(),
        },
        InstallPlanOperation::UnmountTarget {
            mountpoint: "/mnt/aqua-target",
        },
    ];
    let blockers = Vec::new();
    let fingerprint = plan_fingerprint(model.mode, &device, &operations, &blockers);
    Ok(InstallPlan {
        version: INSTALL_PLAN_VERSION,
        source_mode: model.mode,
        target_device: device,
        operations,
        blockers,
        fingerprint,
    })
}

fn partition_path(device: &str, number: u8) -> String {
    let separator = if device.as_bytes().last().is_some_and(u8::is_ascii_digit) {
        "p"
    } else {
        ""
    };
    format!("{device}{separator}{number}")
}

fn validate_plan_path(path: &Path, field: &'static str) -> Result<(), InstallPlanError> {
    let value = path.to_string_lossy();
    if !path.is_absolute()
        || value.len() > 4096
        || value.chars().any(char::is_control)
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(InstallPlanError::InvalidArtifactPath(field));
    }
    Ok(())
}

fn encode_plan_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/' | b':') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

fn plan_fingerprint(
    mode: InstallMode,
    target: &str,
    operations: &[InstallPlanOperation],
    blockers: &[InstallPlanBlocker],
) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut update = |value: &str| {
        for byte in value.bytes().chain(std::iter::once(b'\n')) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    };
    update(match mode {
        InstallMode::DryRun => "dry-run",
        InstallMode::Real => "real",
    });
    update(target);
    for operation in operations {
        update(&operation.render());
    }
    for blocker in blockers {
        update(blocker.id());
    }
    hash
}

fn validate_bounded_value(
    field: &'static str,
    value: String,
    max_len: usize,
) -> Result<String, InstallerError> {
    if value.trim().is_empty() || value.len() > max_len || contains_control(&value) {
        return Err(InstallerError::InvalidValue(field));
    }
    Ok(value)
}

fn contains_control(value: &str) -> bool {
    value.chars().any(char::is_control)
}

pub const STORAGE_PROBE_STATUS: &str = "bounded-storage-probe-ready";
pub const STORAGE_DEVICE_LIMIT: usize = 64;
pub const STORAGE_METADATA_LIMIT: u64 = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageProbePaths {
    pub sys_class_block: PathBuf,
    pub proc_mountinfo: PathBuf,
    pub proc_cmdline: PathBuf,
}

impl StorageProbePaths {
    pub fn system() -> Self {
        Self {
            sys_class_block: PathBuf::from("/sys/class/block"),
            proc_mountinfo: PathBuf::from("/proc/self/mountinfo"),
            proc_cmdline: PathBuf::from("/proc/cmdline"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBlockReason {
    RunningSystemDisk,
    ReadOnly,
    ZeroCapacity,
}

impl StorageBlockReason {
    pub const fn id(self) -> &'static str {
        match self {
            Self::RunningSystemDisk => "running-system-disk",
            Self::ReadOnly => "read-only",
            Self::ZeroCapacity => "zero-capacity",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageCandidate {
    device: String,
    stable_id: String,
    model: String,
    capacity_bytes: u64,
    removable: bool,
    blocked_reasons: Vec<StorageBlockReason>,
}

impl StorageCandidate {
    pub fn device(&self) -> &str {
        &self.device
    }

    pub fn stable_id(&self) -> &str {
        &self.stable_id
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub const fn capacity_bytes(&self) -> u64 {
        self.capacity_bytes
    }

    pub const fn removable(&self) -> bool {
        self.removable
    }

    pub fn blocked_reasons(&self) -> &[StorageBlockReason] {
        &self.blocked_reasons
    }

    pub fn is_eligible(&self) -> bool {
        self.blocked_reasons.is_empty()
    }

    pub fn into_erase_target(self) -> Result<InstallTarget, StorageProbeError> {
        if !self.is_eligible() {
            return Err(StorageProbeError::BlockedTarget {
                device: self.device,
                reasons: self.blocked_reasons,
            });
        }
        let identity =
            DiskIdentity::new(self.device, self.stable_id, self.model, self.capacity_bytes)
                .map_err(|_| StorageProbeError::InvalidMetadata {
                    device: "selected-target".to_string(),
                    field: "identity",
                })?;
        Ok(InstallTarget::erase_disk(identity))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageInventory {
    candidates: Vec<StorageCandidate>,
    root_device_names: Vec<String>,
    root_major_minor: Vec<String>,
}

impl StorageInventory {
    pub fn candidates(&self) -> &[StorageCandidate] {
        &self.candidates
    }

    pub fn root_device_names(&self) -> &[String] {
        &self.root_device_names
    }

    pub fn root_major_minor(&self) -> &[String] {
        &self.root_major_minor
    }

    pub fn eligible_candidates(&self) -> impl Iterator<Item = &StorageCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.is_eligible())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageProbeError {
    Io {
        context: &'static str,
        kind: io::ErrorKind,
        message: String,
    },
    MetadataTooLarge(&'static str),
    TooManyDevices(usize),
    InvalidMetadata {
        device: String,
        field: &'static str,
    },
    BlockedTarget {
        device: String,
        reasons: Vec<StorageBlockReason>,
    },
}

impl fmt::Display for StorageProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                context, message, ..
            } => write!(formatter, "{context}: {message}"),
            Self::MetadataTooLarge(context) => write!(formatter, "{context} exceeds size limit"),
            Self::TooManyDevices(count) => {
                write!(formatter, "block device count {count} exceeds limit")
            }
            Self::InvalidMetadata { device, field } => {
                write!(formatter, "invalid {field} metadata for {device}")
            }
            Self::BlockedTarget { device, reasons } => {
                let reason_ids = reasons
                    .iter()
                    .map(|reason| reason.id())
                    .collect::<Vec<_>>()
                    .join(",");
                write!(formatter, "blocked install target {device}: {reason_ids}")
            }
        }
    }
}

impl Error for StorageProbeError {}

pub fn probe_storage(paths: &StorageProbePaths) -> Result<StorageInventory, StorageProbeError> {
    let mountinfo = read_bounded_file(&paths.proc_mountinfo, "proc-mountinfo")?;
    let cmdline =
        read_optional_bounded_file(&paths.proc_cmdline, "proc-cmdline")?.unwrap_or_default();
    let mut root_devices = root_devices_from_mountinfo(&mountinfo);
    root_devices
        .device_names
        .extend(root_devices_from_cmdline(&cmdline));
    root_devices.normalize();

    let mut entries = fs::read_dir(&paths.sys_class_block)
        .map_err(|error| storage_io("read-sys-class-block", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| storage_io("enumerate-sys-class-block", error))?;
    if entries.len() > STORAGE_DEVICE_LIMIT {
        return Err(StorageProbeError::TooManyDevices(entries.len()));
    }
    entries.sort_by_key(|entry| entry.file_name());

    let entry_paths = entries
        .iter()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            valid_block_name(&name).then_some((name, entry.path()))
        })
        .collect::<Vec<_>>();

    let mut candidates = Vec::new();
    for (name, path) in &entry_paths {
        if is_pseudo_block_device(name) || path.join("partition").exists() {
            continue;
        }
        if let Some(uevent) = read_optional_bounded_file(&path.join("uevent"), "block-uevent")? {
            if uevent.lines().any(|line| line == "DEVTYPE=partition") {
                continue;
            }
        }

        let sectors = read_required_u64(&path.join("size"), name, "size")?;
        let capacity_bytes =
            sectors
                .checked_mul(512)
                .ok_or_else(|| StorageProbeError::InvalidMetadata {
                    device: name.clone(),
                    field: "size",
                })?;
        let major_minor = read_required_value(&path.join("dev"), name, "dev")?;
        if !valid_major_minor(&major_minor) {
            return Err(StorageProbeError::InvalidMetadata {
                device: name.clone(),
                field: "dev",
            });
        }
        let model = read_optional_value(&path.join("device/model"), "device-model")?
            .unwrap_or_else(|| "Unknown block device".to_string());
        let stable_id = read_optional_value(&path.join("device/wwid"), "device-wwid")?
            .or(read_optional_value(
                &path.join("device/serial"),
                "device-serial",
            )?)
            .unwrap_or_else(|| format!("dev-{major_minor}-{name}"));
        let read_only = read_optional_flag(&path.join("ro"), "device-read-only")?.unwrap_or(false);
        let removable =
            read_optional_flag(&path.join("removable"), "device-removable")?.unwrap_or(false);

        DiskIdentity::new(
            format!("/dev/{name}"),
            stable_id.clone(),
            model.clone(),
            capacity_bytes.max(1),
        )
        .map_err(|_| StorageProbeError::InvalidMetadata {
            device: name.clone(),
            field: "identity",
        })?;
        let mut blocked_reasons = Vec::new();
        if capacity_bytes == 0 {
            blocked_reasons.push(StorageBlockReason::ZeroCapacity);
        }
        if read_only {
            blocked_reasons.push(StorageBlockReason::ReadOnly);
        }
        if candidate_contains_root_device(name, path, &major_minor, &entry_paths, &root_devices) {
            blocked_reasons.push(StorageBlockReason::RunningSystemDisk);
        }
        candidates.push(StorageCandidate {
            device: format!("/dev/{name}"),
            stable_id,
            model,
            capacity_bytes,
            removable,
            blocked_reasons,
        });
    }

    Ok(StorageInventory {
        candidates,
        root_device_names: root_devices.device_names,
        root_major_minor: root_devices.major_minor,
    })
}

#[derive(Debug, Default)]
struct RootDevices {
    device_names: Vec<String>,
    major_minor: Vec<String>,
}

impl RootDevices {
    fn normalize(&mut self) {
        self.device_names.sort();
        self.device_names.dedup();
        self.major_minor.sort();
        self.major_minor.dedup();
    }
}

fn root_devices_from_mountinfo(mountinfo: &str) -> RootDevices {
    let mut roots = RootDevices::default();
    for line in mountinfo.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let Some(separator) = fields.iter().position(|field| *field == "-") else {
            continue;
        };
        if fields.get(4) != Some(&"/") {
            continue;
        }
        if let Some(major_minor) = fields.get(2).filter(|value| valid_major_minor(value)) {
            if *major_minor != "0:0" {
                roots.major_minor.push((*major_minor).to_string());
            }
        }
        if let Some(source) = fields
            .get(separator + 2)
            .and_then(|source| device_basename(source))
        {
            roots.device_names.push(source.to_string());
        }
    }
    roots
}

fn root_devices_from_cmdline(cmdline: &str) -> Vec<String> {
    cmdline
        .split_whitespace()
        .filter_map(|value| value.strip_prefix("root="))
        .filter_map(device_basename)
        .map(str::to_string)
        .collect()
}

fn device_basename(value: &str) -> Option<&str> {
    let name = value.strip_prefix("/dev/")?;
    valid_block_name(name).then_some(name)
}

fn candidate_contains_root_device(
    candidate_name: &str,
    candidate_path: &Path,
    candidate_major_minor: &str,
    entries: &[(String, PathBuf)],
    roots: &RootDevices,
) -> bool {
    if roots.device_names.iter().any(|name| name == candidate_name)
        || roots
            .major_minor
            .iter()
            .any(|value| value == candidate_major_minor)
    {
        return true;
    }
    let candidate_real = fs::canonicalize(candidate_path).ok();
    entries.iter().any(|(name, path)| {
        if !path.join("partition").exists() {
            return false;
        }
        let name_matches = roots.device_names.iter().any(|root| root == name);
        let dev_matches = read_optional_value(&path.join("dev"), "partition-dev")
            .ok()
            .flatten()
            .is_some_and(|value| roots.major_minor.iter().any(|root| root == &value));
        if !name_matches && !dev_matches {
            return false;
        }
        match (&candidate_real, fs::canonicalize(path).ok()) {
            (Some(candidate), Some(partition)) => partition.starts_with(candidate),
            _ => false,
        }
    })
}

fn read_required_u64(
    path: &Path,
    device: &str,
    field: &'static str,
) -> Result<u64, StorageProbeError> {
    read_required_value(path, device, field)?
        .parse::<u64>()
        .map_err(|_| StorageProbeError::InvalidMetadata {
            device: device.to_string(),
            field,
        })
}

fn read_required_value(
    path: &Path,
    device: &str,
    field: &'static str,
) -> Result<String, StorageProbeError> {
    read_optional_value(path, field)?.ok_or_else(|| StorageProbeError::InvalidMetadata {
        device: device.to_string(),
        field,
    })
}

fn read_optional_flag(
    path: &Path,
    context: &'static str,
) -> Result<Option<bool>, StorageProbeError> {
    read_optional_value(path, context)?
        .map(|value| match value.as_str() {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(StorageProbeError::InvalidMetadata {
                device: path
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                field: context,
            }),
        })
        .transpose()
}

fn read_optional_value(
    path: &Path,
    context: &'static str,
) -> Result<Option<String>, StorageProbeError> {
    Ok(read_optional_bounded_file(path, context)?
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()))
}

fn read_optional_bounded_file(
    path: &Path,
    context: &'static str,
) -> Result<Option<String>, StorageProbeError> {
    match read_bounded_file(path, context) {
        Ok(value) => Ok(Some(value)),
        Err(StorageProbeError::Io {
            kind: io::ErrorKind::NotFound,
            ..
        }) => Ok(None),
        Err(error) => Err(error),
    }
}

fn read_bounded_file(path: &Path, context: &'static str) -> Result<String, StorageProbeError> {
    let file = File::open(path).map_err(|error| storage_io(context, error))?;
    let mut bytes = Vec::new();
    file.take(STORAGE_METADATA_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| storage_io(context, error))?;
    if bytes.len() as u64 > STORAGE_METADATA_LIMIT {
        return Err(StorageProbeError::MetadataTooLarge(context));
    }
    String::from_utf8(bytes).map_err(|_| StorageProbeError::InvalidMetadata {
        device: "system".to_string(),
        field: context,
    })
}

fn storage_io(context: &'static str, error: io::Error) -> StorageProbeError {
    StorageProbeError::Io {
        context,
        kind: error.kind(),
        message: error.to_string(),
    }
}

fn valid_block_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_major_minor(value: &str) -> bool {
    let mut parts = value.split(':');
    matches!(parts.next(), Some(part) if !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && matches!(parts.next(), Some(part) if !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
}

fn is_pseudo_block_device(name: &str) -> bool {
    ["loop", "ram", "zram", "fd", "sr"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

pub const INSTALL_PREREQUISITES_STATUS: &str = "installer-prerequisites-ready";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTool {
    Sfdisk,
    MkfsFat,
    MkfsExt4,
    Tar,
    Mount,
    Umount,
}

impl InstallTool {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Sfdisk => "sfdisk",
            Self::MkfsFat => "mkfs-fat",
            Self::MkfsExt4 => "mkfs-ext4",
            Self::Tar => "tar",
            Self::Mount => "mount",
            Self::Umount => "umount",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallToolPaths {
    pub sfdisk: PathBuf,
    pub mkfs_fat: PathBuf,
    pub mkfs_ext4: PathBuf,
    pub tar: PathBuf,
    pub mount: PathBuf,
    pub umount: PathBuf,
}

impl InstallToolPaths {
    pub fn system() -> Self {
        Self {
            sfdisk: PathBuf::from("/sbin/sfdisk"),
            mkfs_fat: PathBuf::from("/sbin/mkfs.fat"),
            mkfs_ext4: PathBuf::from("/sbin/mkfs.ext4"),
            tar: PathBuf::from("/bin/tar"),
            mount: PathBuf::from("/bin/mount"),
            umount: PathBuf::from("/bin/umount"),
        }
    }

    fn entries(&self) -> [(InstallTool, &Path); 6] {
        [
            (InstallTool::Sfdisk, &self.sfdisk),
            (InstallTool::MkfsFat, &self.mkfs_fat),
            (InstallTool::MkfsExt4, &self.mkfs_ext4),
            (InstallTool::Tar, &self.tar),
            (InstallTool::Mount, &self.mount),
            (InstallTool::Umount, &self.umount),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedInstallTool {
    tool: InstallTool,
    path: PathBuf,
}

impl ValidatedInstallTool {
    pub const fn tool(&self) -> InstallTool {
        self.tool
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPrerequisites {
    tools: Vec<ValidatedInstallTool>,
}

impl InstallPrerequisites {
    pub fn tools(&self) -> &[ValidatedInstallTool] {
        &self.tools
    }

    fn path(&self, tool: InstallTool) -> Option<&Path> {
        self.tools
            .iter()
            .find(|candidate| candidate.tool == tool)
            .map(|candidate| candidate.path.as_path())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPrerequisiteError {
    InvalidPath(InstallTool),
    Missing(InstallTool),
    NotExecutable(InstallTool),
    Io {
        tool: InstallTool,
        kind: io::ErrorKind,
        message: String,
    },
}

impl fmt::Display for InstallPrerequisiteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(tool) => write!(formatter, "invalid {} path", tool.id()),
            Self::Missing(tool) => write!(formatter, "missing installer tool {}", tool.id()),
            Self::NotExecutable(tool) => {
                write!(formatter, "installer tool {} is not executable", tool.id())
            }
            Self::Io { tool, message, .. } => write!(formatter, "{}: {message}", tool.id()),
        }
    }
}

impl Error for InstallPrerequisiteError {}

pub fn validate_install_prerequisites(
    paths: &InstallToolPaths,
) -> Result<InstallPrerequisites, InstallPrerequisiteError> {
    let mut tools = Vec::with_capacity(paths.entries().len());
    for (tool, path) in paths.entries() {
        if !path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(InstallPrerequisiteError::InvalidPath(tool));
        }
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(InstallPrerequisiteError::Missing(tool));
            }
            Err(error) => {
                return Err(InstallPrerequisiteError::Io {
                    tool,
                    kind: error.kind(),
                    message: error.to_string(),
                });
            }
        };
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            return Err(InstallPrerequisiteError::NotExecutable(tool));
        }
        tools.push(ValidatedInstallTool {
            tool,
            path: path.to_path_buf(),
        });
    }
    Ok(InstallPrerequisites { tools })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetRevalidationError {
    Probe(StorageProbeError),
    Missing(String),
    IdentityChanged {
        device: String,
        field: &'static str,
    },
    Blocked {
        device: String,
        reasons: Vec<StorageBlockReason>,
    },
}

impl fmt::Display for TargetRevalidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Probe(error) => write!(formatter, "target revalidation probe failed: {error}"),
            Self::Missing(device) => write!(formatter, "install target disappeared: {device}"),
            Self::IdentityChanged { device, field } => {
                write!(formatter, "install target {device} changed {field}")
            }
            Self::Blocked { device, reasons } => {
                let reason_ids = reasons
                    .iter()
                    .map(|reason| reason.id())
                    .collect::<Vec<_>>()
                    .join(",");
                write!(
                    formatter,
                    "install target {device} became blocked: {reason_ids}"
                )
            }
        }
    }
}

impl Error for TargetRevalidationError {}

pub fn revalidate_install_target(
    paths: &StorageProbePaths,
    expected: &DiskIdentity,
) -> Result<StorageCandidate, TargetRevalidationError> {
    let inventory = probe_storage(paths).map_err(TargetRevalidationError::Probe)?;
    let candidate = inventory
        .candidates()
        .iter()
        .find(|candidate| candidate.device() == expected.device())
        .cloned()
        .ok_or_else(|| TargetRevalidationError::Missing(expected.device().to_string()))?;
    for (field, matches) in [
        ("stable-id", candidate.stable_id() == expected.stable_id()),
        ("model", candidate.model() == expected.model()),
        (
            "capacity-bytes",
            candidate.capacity_bytes() == expected.capacity_bytes(),
        ),
    ] {
        if !matches {
            return Err(TargetRevalidationError::IdentityChanged {
                device: expected.device().to_string(),
                field,
            });
        }
    }
    if !candidate.is_eligible() {
        return Err(TargetRevalidationError::Blocked {
            device: expected.device().to_string(),
            reasons: candidate.blocked_reasons().to_vec(),
        });
    }
    Ok(candidate)
}

pub const INSTALL_COMMAND_PLAN_STATUS: &str = "bounded-command-plan-ready";
pub const INSTALL_COMMAND_REHEARSAL_STATUS: &str = "non-executing-command-rehearsal-ready";
pub const INSTALL_COMMAND_LIMIT: usize = 16;
pub const INSTALL_ARGUMENT_LIMIT: usize = 32;
pub const INSTALL_ARGUMENT_BYTES_LIMIT: usize = 4096;
pub const INSTALL_STDIN_BYTES_LIMIT: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallCommandSpec {
    operation: &'static str,
    tool: InstallTool,
    program: PathBuf,
    arguments: Vec<String>,
    stdin: Option<String>,
}

impl InstallCommandSpec {
    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    pub const fn tool(&self) -> InstallTool {
        self.tool
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub fn stdin(&self) -> Option<&str> {
        self.stdin.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallCommandPlan {
    source_fingerprint: u64,
    commands: Vec<InstallCommandSpec>,
    deferred_operations: Vec<&'static str>,
}

impl InstallCommandPlan {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub fn commands(&self) -> &[InstallCommandSpec] {
        &self.commands
    }

    pub fn deferred_operations(&self) -> &[&'static str] {
        &self.deferred_operations
    }

    pub const fn execution_allowed(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallCommandCompileError {
    MissingTool(InstallTool),
    InvalidPlan(&'static str),
    UnsafeArgument(&'static str),
    UnsafeStdin,
    TooManyCommands,
}

impl fmt::Display for InstallCommandCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTool(tool) => write!(formatter, "missing validated tool {}", tool.id()),
            Self::InvalidPlan(reason) => write!(formatter, "invalid command plan: {reason}"),
            Self::UnsafeArgument(operation) => {
                write!(formatter, "unsafe argument for operation {operation}")
            }
            Self::UnsafeStdin => formatter.write_str("unsafe bounded stdin payload"),
            Self::TooManyCommands => formatter.write_str("installer command limit exceeded"),
        }
    }
}

impl Error for InstallCommandCompileError {}

fn install_command(
    prerequisites: &InstallPrerequisites,
    operation: &'static str,
    tool: InstallTool,
    arguments: Vec<String>,
    stdin: Option<String>,
) -> Result<InstallCommandSpec, InstallCommandCompileError> {
    if arguments.len() > INSTALL_ARGUMENT_LIMIT
        || arguments.iter().any(|argument| {
            argument.is_empty()
                || argument.len() > INSTALL_ARGUMENT_BYTES_LIMIT
                || argument.chars().any(char::is_control)
        })
    {
        return Err(InstallCommandCompileError::UnsafeArgument(operation));
    }
    if stdin.as_ref().is_some_and(|payload| {
        payload.len() > INSTALL_STDIN_BYTES_LIMIT || payload.bytes().any(|byte| byte == 0)
    }) {
        return Err(InstallCommandCompileError::UnsafeStdin);
    }
    let program = prerequisites
        .path(tool)
        .ok_or(InstallCommandCompileError::MissingTool(tool))?
        .to_path_buf();
    Ok(InstallCommandSpec {
        operation,
        tool,
        program,
        arguments,
        stdin,
    })
}

pub fn compile_install_commands(
    plan: &InstallPlan,
    prerequisites: &InstallPrerequisites,
) -> Result<InstallCommandPlan, InstallCommandCompileError> {
    let operations = plan.operations();
    if operations.len() != 13 {
        return Err(InstallCommandCompileError::InvalidPlan(
            "canonical operation count changed",
        ));
    }
    let InstallPlanOperation::VerifyTarget {
        device: verified_device,
        ..
    } = &operations[0]
    else {
        return Err(InstallCommandCompileError::InvalidPlan(
            "target verification is not first",
        ));
    };
    let (
        InstallPlanOperation::WriteGpt { device },
        InstallPlanOperation::CreateEfiSystemPartition {
            partition: efi_partition,
            start_mib: efi_start_mib,
            size_mib: efi_size_mib,
        },
        InstallPlanOperation::CreateRootPartition {
            partition: root_partition,
            start_mib: root_start_mib,
            filesystem: Filesystem::Ext4,
        },
    ) = (&operations[1], &operations[2], &operations[3])
    else {
        return Err(InstallCommandCompileError::InvalidPlan(
            "partition operations are not canonical",
        ));
    };
    if verified_device != plan.target_device()
        || device != plan.target_device()
        || efi_partition != &partition_path(device, 1)
        || root_partition != &partition_path(device, 2)
        || *efi_start_mib != INSTALL_ESP_START_MIB
        || *efi_size_mib != INSTALL_ESP_SIZE_MIB
        || *root_start_mib != INSTALL_ESP_START_MIB + INSTALL_ESP_SIZE_MIB
    {
        return Err(InstallCommandCompileError::InvalidPlan(
            "partition geometry or identity changed",
        ));
    }
    let canonical_tail = matches!(
        (&operations[4], &operations[5], &operations[6], &operations[7], &operations[8], &operations[9], &operations[10], &operations[11], &operations[12]),
        (
            InstallPlanOperation::FormatEfiSystemPartition { partition: format_efi, label: INSTALL_ESP_LABEL },
            InstallPlanOperation::FormatRootPartition { partition: format_root, filesystem: Filesystem::Ext4, label: INSTALL_ROOT_LABEL },
            InstallPlanOperation::MountRoot { partition: mount_root, mountpoint: "/mnt/aqua-target" },
            InstallPlanOperation::ExtractRootFilesystem { destination: "/mnt/aqua-target", .. },
            InstallPlanOperation::MountEfiSystemPartition { partition: mount_efi, mountpoint: "/mnt/aqua-target/boot/efi" },
            InstallPlanOperation::InstallKernel { destination: "/mnt/aqua-target/boot/vmlinuz-aqua", .. },
            InstallPlanOperation::InstallBootloader { strategy: BootloaderStrategy::Grub2X86_64Efi, destination: "/mnt/aqua-target/boot/efi/EFI/BOOT/BOOTX64.EFI", config_destination: "/mnt/aqua-target/boot/efi/EFI/BOOT/grub.cfg", root_label: INSTALL_ROOT_LABEL, kernel_path: "/boot/vmlinuz-aqua", .. },
            InstallPlanOperation::WriteSystemConfiguration { .. },
            InstallPlanOperation::UnmountTarget { mountpoint: "/mnt/aqua-target" },
        ) if format_efi == efi_partition
            && format_root == root_partition
            && mount_root == root_partition
            && mount_efi == efi_partition
    );
    if !canonical_tail {
        return Err(InstallCommandCompileError::InvalidPlan(
            "command and internal operation order changed",
        ));
    }

    const SECTORS_PER_MIB: u64 = 2048;
    let efi_start_sector = efi_start_mib * SECTORS_PER_MIB;
    let efi_size_sectors = efi_size_mib * SECTORS_PER_MIB;
    let root_start_sector = root_start_mib * SECTORS_PER_MIB;
    let partition_recipe = format!(
        "label: gpt\nunit: sectors\n\nstart={efi_start_sector}, size={efi_size_sectors}, type=U, name=\"AQUA_EFI\"\nstart={root_start_sector}, type=L, name=\"AQUA_ROOT\"\n"
    );
    let mut commands = vec![install_command(
        prerequisites,
        "write-partition-table",
        InstallTool::Sfdisk,
        vec![
            "--wipe".to_string(),
            "always".to_string(),
            "--wipe-partitions".to_string(),
            "always".to_string(),
            device.clone(),
        ],
        Some(partition_recipe),
    )?];

    for operation in &operations[4..] {
        match operation {
            InstallPlanOperation::FormatEfiSystemPartition { partition, label } => {
                commands.push(install_command(
                    prerequisites,
                    operation.id(),
                    InstallTool::MkfsFat,
                    vec![
                        "-F".to_string(),
                        "32".to_string(),
                        "-n".to_string(),
                        (*label).to_string(),
                        partition.clone(),
                    ],
                    None,
                )?);
            }
            InstallPlanOperation::FormatRootPartition {
                partition,
                filesystem: Filesystem::Ext4,
                label,
            } => commands.push(install_command(
                prerequisites,
                operation.id(),
                InstallTool::MkfsExt4,
                vec![
                    "-F".to_string(),
                    "-L".to_string(),
                    (*label).to_string(),
                    partition.clone(),
                ],
                None,
            )?),
            InstallPlanOperation::MountRoot {
                partition,
                mountpoint,
            }
            | InstallPlanOperation::MountEfiSystemPartition {
                partition,
                mountpoint,
            } => commands.push(install_command(
                prerequisites,
                operation.id(),
                InstallTool::Mount,
                vec![partition.clone(), (*mountpoint).to_string()],
                None,
            )?),
            InstallPlanOperation::ExtractRootFilesystem {
                source,
                destination,
            } => commands.push(install_command(
                prerequisites,
                operation.id(),
                InstallTool::Tar,
                vec![
                    "--extract".to_string(),
                    "--file".to_string(),
                    source.to_string_lossy().into_owned(),
                    "--directory".to_string(),
                    (*destination).to_string(),
                    "--numeric-owner".to_string(),
                    "--same-owner".to_string(),
                ],
                None,
            )?),
            InstallPlanOperation::UnmountTarget { mountpoint } => {
                commands.push(install_command(
                    prerequisites,
                    operation.id(),
                    InstallTool::Umount,
                    vec![format!("{mountpoint}/boot/efi")],
                    None,
                )?);
                commands.push(install_command(
                    prerequisites,
                    operation.id(),
                    InstallTool::Umount,
                    vec![(*mountpoint).to_string()],
                    None,
                )?);
            }
            InstallPlanOperation::InstallKernel { .. }
            | InstallPlanOperation::InstallBootloader { .. }
            | InstallPlanOperation::WriteSystemConfiguration { .. } => {}
            _ => {
                return Err(InstallCommandCompileError::InvalidPlan(
                    "unexpected operation in command section",
                ));
            }
        }
    }
    if commands.len() > INSTALL_COMMAND_LIMIT {
        return Err(InstallCommandCompileError::TooManyCommands);
    }
    Ok(InstallCommandPlan {
        source_fingerprint: plan.fingerprint(),
        commands,
        deferred_operations: vec![
            "verify-target",
            "install-kernel",
            "install-bootloader",
            "write-system-configuration",
        ],
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NonExecutingInstallCommandRunner {
    rehearsed: Vec<InstallCommandSpec>,
}

impl NonExecutingInstallCommandRunner {
    pub fn rehearse(&mut self, plan: &InstallCommandPlan) -> InstallCommandRehearsal {
        self.rehearsed.extend(plan.commands.iter().cloned());
        InstallCommandRehearsal {
            source_fingerprint: plan.source_fingerprint,
            command_count: plan.commands.len(),
        }
    }

    pub fn rehearsed(&self) -> &[InstallCommandSpec] {
        &self.rehearsed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallCommandRehearsal {
    source_fingerprint: u64,
    command_count: usize,
}

impl InstallCommandRehearsal {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub const fn command_count(&self) -> usize {
        self.command_count
    }

    pub const fn executed(&self) -> bool {
        false
    }
}

pub const INTERNAL_INSTALL_PLAN_STATUS: &str = "bounded-internal-install-plan-ready";
pub const INTERNAL_INSTALL_REHEARSAL_STATUS: &str =
    "non-executing-internal-install-rehearsal-ready";
pub const INTERNAL_INSTALL_ACTION_LIMIT: usize = 16;
pub const INTERNAL_INSTALL_CONTENT_LIMIT: usize = 16 * 1024;
const INSTALL_TARGET_ROOT: &str = "/mnt/aqua-target";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalInstallAnchor {
    BeforeMountRoot,
    BeforeMountEfi,
    AfterMountEfi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalInstallActionKind {
    CreateDirectory {
        path: PathBuf,
        mode: u32,
    },
    CopyFileAtomic {
        source: PathBuf,
        destination: PathBuf,
        temporary: PathBuf,
        mode: u32,
    },
    WriteFileAtomic {
        destination: PathBuf,
        temporary: PathBuf,
        content: String,
        mode: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalInstallAction {
    anchor: InternalInstallAnchor,
    kind: InternalInstallActionKind,
}

impl InternalInstallAction {
    pub const fn anchor(&self) -> InternalInstallAnchor {
        self.anchor
    }

    pub const fn kind(&self) -> &InternalInstallActionKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalInstallPlan {
    source_fingerprint: u64,
    actions: Vec<InternalInstallAction>,
}

impl InternalInstallPlan {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub fn actions(&self) -> &[InternalInstallAction] {
        &self.actions
    }

    pub const fn execution_allowed(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalInstallCompileError {
    InvalidPlan(&'static str),
    UnsafePath(&'static str),
    UnsafeContent(&'static str),
    TooManyActions,
}

impl fmt::Display for InternalInstallCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlan(reason) => write!(formatter, "invalid internal plan: {reason}"),
            Self::UnsafePath(field) => write!(formatter, "unsafe internal path: {field}"),
            Self::UnsafeContent(field) => write!(formatter, "unsafe internal content: {field}"),
            Self::TooManyActions => formatter.write_str("internal install action limit exceeded"),
        }
    }
}

impl Error for InternalInstallCompileError {}

fn validate_internal_destination(
    path: &Path,
    field: &'static str,
) -> Result<(), InternalInstallCompileError> {
    if !path.is_absolute()
        || !path.starts_with(INSTALL_TARGET_ROOT)
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        || path.to_string_lossy().len() > INSTALL_ARGUMENT_BYTES_LIMIT
        || path.to_string_lossy().chars().any(char::is_control)
    {
        return Err(InternalInstallCompileError::UnsafePath(field));
    }
    Ok(())
}

fn atomic_temporary_path(
    destination: &Path,
    field: &'static str,
) -> Result<PathBuf, InternalInstallCompileError> {
    validate_internal_destination(destination, field)?;
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(InternalInstallCompileError::UnsafePath(field))?;
    Ok(destination.with_file_name(format!(".{file_name}.aqua-install.tmp")))
}

fn write_action(
    anchor: InternalInstallAnchor,
    destination: impl Into<PathBuf>,
    content: String,
    field: &'static str,
) -> Result<InternalInstallAction, InternalInstallCompileError> {
    if content.len() > INTERNAL_INSTALL_CONTENT_LIMIT || content.bytes().any(|byte| byte == 0) {
        return Err(InternalInstallCompileError::UnsafeContent(field));
    }
    let destination = destination.into();
    let temporary = atomic_temporary_path(&destination, field)?;
    Ok(InternalInstallAction {
        anchor,
        kind: InternalInstallActionKind::WriteFileAtomic {
            destination,
            temporary,
            content,
            mode: 0o644,
        },
    })
}

fn copy_action(
    anchor: InternalInstallAnchor,
    source: &Path,
    destination: impl Into<PathBuf>,
    field: &'static str,
) -> Result<InternalInstallAction, InternalInstallCompileError> {
    validate_plan_path(source, field)
        .map_err(|_| InternalInstallCompileError::UnsafePath(field))?;
    let destination = destination.into();
    let temporary = atomic_temporary_path(&destination, field)?;
    Ok(InternalInstallAction {
        anchor,
        kind: InternalInstallActionKind::CopyFileAtomic {
            source: source.to_path_buf(),
            destination,
            temporary,
            mode: 0o644,
        },
    })
}

pub fn compile_internal_install_actions(
    plan: &InstallPlan,
) -> Result<InternalInstallPlan, InternalInstallCompileError> {
    if plan.operations().len() != 13 {
        return Err(InternalInstallCompileError::InvalidPlan(
            "canonical operation count changed",
        ));
    }
    let (
        InstallPlanOperation::MountRoot { mountpoint, .. },
        InstallPlanOperation::ExtractRootFilesystem { destination, .. },
        InstallPlanOperation::MountEfiSystemPartition {
            mountpoint: efi_mountpoint,
            ..
        },
        InstallPlanOperation::InstallKernel {
            source: kernel_source,
            destination: kernel_destination,
        },
        InstallPlanOperation::InstallBootloader {
            strategy: BootloaderStrategy::Grub2X86_64Efi,
            source: bootloader_source,
            destination: bootloader_destination,
            config_destination,
            root_label,
            kernel_path,
            kernel_cmdline,
        },
        InstallPlanOperation::WriteSystemConfiguration {
            locale,
            keyboard_layout,
            timezone,
            username,
            display_name,
            password_configured,
        },
        InstallPlanOperation::UnmountTarget {
            mountpoint: unmountpoint,
        },
    ) = (
        &plan.operations()[6],
        &plan.operations()[7],
        &plan.operations()[8],
        &plan.operations()[9],
        &plan.operations()[10],
        &plan.operations()[11],
        &plan.operations()[12],
    )
    else {
        return Err(InternalInstallCompileError::InvalidPlan(
            "internal operation order changed",
        ));
    };
    if *mountpoint != INSTALL_TARGET_ROOT
        || *destination != INSTALL_TARGET_ROOT
        || *efi_mountpoint != "/mnt/aqua-target/boot/efi"
        || *unmountpoint != INSTALL_TARGET_ROOT
        || *root_label != INSTALL_ROOT_LABEL
        || !*password_configured
    {
        return Err(InternalInstallCompileError::InvalidPlan(
            "internal destination or account contract changed",
        ));
    }

    let grub_configuration = format!(
        "set default=\"0\"\nset timeout=\"3\"\n\nsearch --no-floppy --label {root_label} --set=root\n\nmenuentry \"Aqua Linux\" {{\n\tlinux {kernel_path} {kernel_cmdline}\n}}\n"
    );
    let mut actions = vec![InternalInstallAction {
        anchor: InternalInstallAnchor::BeforeMountRoot,
        kind: InternalInstallActionKind::CreateDirectory {
            path: PathBuf::from(INSTALL_TARGET_ROOT),
            mode: 0o755,
        },
    }];
    actions.push(InternalInstallAction {
        anchor: InternalInstallAnchor::BeforeMountEfi,
        kind: InternalInstallActionKind::CreateDirectory {
            path: PathBuf::from(efi_mountpoint),
            mode: 0o755,
        },
    });
    actions.push(InternalInstallAction {
        anchor: InternalInstallAnchor::AfterMountEfi,
        kind: InternalInstallActionKind::CreateDirectory {
            path: PathBuf::from("/mnt/aqua-target/boot/efi/EFI/BOOT"),
            mode: 0o755,
        },
    });
    actions.push(copy_action(
        InternalInstallAnchor::AfterMountEfi,
        kernel_source,
        *kernel_destination,
        "kernel-copy",
    )?);
    actions.push(copy_action(
        InternalInstallAnchor::AfterMountEfi,
        bootloader_source,
        *bootloader_destination,
        "bootloader-copy",
    )?);
    actions.push(write_action(
        InternalInstallAnchor::AfterMountEfi,
        *config_destination,
        grub_configuration,
        "grub-configuration",
    )?);
    actions.push(InternalInstallAction {
        anchor: InternalInstallAnchor::AfterMountEfi,
        kind: InternalInstallActionKind::CreateDirectory {
            path: PathBuf::from("/mnt/aqua-target/etc/aqua"),
            mode: 0o755,
        },
    });
    actions.push(write_action(
        InternalInstallAnchor::AfterMountEfi,
        "/mnt/aqua-target/etc/locale.conf",
        format!("LANG={locale}\n"),
        "locale-configuration",
    )?);
    actions.push(write_action(
        InternalInstallAnchor::AfterMountEfi,
        "/mnt/aqua-target/etc/vconsole.conf",
        format!("KEYMAP={keyboard_layout}\n"),
        "keyboard-configuration",
    )?);
    actions.push(write_action(
        InternalInstallAnchor::AfterMountEfi,
        "/mnt/aqua-target/etc/timezone",
        format!("{timezone}\n"),
        "timezone-configuration",
    )?);
    actions.push(write_action(
        InternalInstallAnchor::AfterMountEfi,
        "/mnt/aqua-target/etc/aqua/first-user.conf",
        format!(
            "username={}\ndisplay_name={}\npassword_configured=true\n",
            encode_plan_value(username),
            encode_plan_value(display_name),
        ),
        "user-configuration",
    )?);
    if actions.len() > INTERNAL_INSTALL_ACTION_LIMIT {
        return Err(InternalInstallCompileError::TooManyActions);
    }
    for action in &actions {
        match action.kind() {
            InternalInstallActionKind::CreateDirectory { path, .. } => {
                validate_internal_destination(path, "directory")?;
            }
            InternalInstallActionKind::CopyFileAtomic {
                destination,
                temporary,
                ..
            }
            | InternalInstallActionKind::WriteFileAtomic {
                destination,
                temporary,
                ..
            } => {
                validate_internal_destination(destination, "destination")?;
                validate_internal_destination(temporary, "temporary")?;
                if destination.parent() != temporary.parent() || destination == temporary {
                    return Err(InternalInstallCompileError::UnsafePath("atomic-temporary"));
                }
            }
        }
    }
    Ok(InternalInstallPlan {
        source_fingerprint: plan.fingerprint(),
        actions,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NonExecutingInternalInstallRunner {
    rehearsed: Vec<InternalInstallAction>,
}

impl NonExecutingInternalInstallRunner {
    pub fn rehearse(&mut self, plan: &InternalInstallPlan) -> InternalInstallRehearsal {
        self.rehearsed.extend(plan.actions.iter().cloned());
        InternalInstallRehearsal {
            source_fingerprint: plan.source_fingerprint,
            action_count: plan.actions.len(),
        }
    }

    pub fn rehearsed(&self) -> &[InternalInstallAction] {
        &self.rehearsed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalInstallRehearsal {
    source_fingerprint: u64,
    action_count: usize,
}

impl InternalInstallRehearsal {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub const fn action_count(&self) -> usize {
        self.action_count
    }

    pub const fn executed(&self) -> bool {
        false
    }
}

pub const INSTALL_TRANSACTION_GRAPH_STATUS: &str = "ordered-install-transaction-graph-ready";
pub const INSTALL_TRANSACTION_REHEARSAL_STATUS: &str =
    "failure-aware-non-executing-transaction-rehearsal-ready";
pub const INSTALL_PROGRESS_STATUS: &str = "transaction-bound-live-progress-model-ready";
pub const INSTALL_TRANSACTION_STEP_LIMIT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallTransactionStep {
    RevalidateTarget { expected: DiskIdentity },
    Command(InstallCommandSpec),
    Internal(InternalInstallAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallCleanupRequirement {
    EfiMounted,
    RootMounted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallCleanupStep {
    requirement: InstallCleanupRequirement,
    command: InstallCommandSpec,
}

impl InstallCleanupStep {
    pub const fn requirement(&self) -> InstallCleanupRequirement {
        self.requirement
    }

    pub const fn command(&self) -> &InstallCommandSpec {
        &self.command
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTransactionGraph {
    source_fingerprint: u64,
    steps: Vec<InstallTransactionStep>,
    cleanup: Vec<InstallCleanupStep>,
}

impl InstallTransactionGraph {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub fn steps(&self) -> &[InstallTransactionStep] {
        &self.steps
    }

    pub fn cleanup(&self) -> &[InstallCleanupStep] {
        &self.cleanup
    }

    pub const fn execution_allowed(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallProgressState {
    Running,
    Completed,
    Failed,
}

impl InstallProgressState {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallProgressPhase {
    PreparingTarget,
    Partitioning,
    Formatting,
    InstallingSystem,
    InstallingBootloader,
    ConfiguringSystem,
    Finalizing,
    Completed,
}

impl InstallProgressPhase {
    pub const fn id(self) -> &'static str {
        match self {
            Self::PreparingTarget => "preparing-target",
            Self::Partitioning => "partitioning",
            Self::Formatting => "formatting",
            Self::InstallingSystem => "installing-system",
            Self::InstallingBootloader => "installing-bootloader",
            Self::ConfiguringSystem => "configuring-system",
            Self::Finalizing => "finalizing",
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallProgressEvent {
    state: InstallProgressState,
    phase: InstallProgressPhase,
    operation: &'static str,
    completed_steps: usize,
    total_steps: usize,
    percent: u8,
}

impl InstallProgressEvent {
    pub fn running(
        graph: &InstallTransactionGraph,
        completed_steps: usize,
    ) -> Result<Self, InstallProgressError> {
        if completed_steps >= graph.steps.len() {
            return Err(InstallProgressError::InvalidCompletedStepCount {
                completed: completed_steps,
                total: graph.steps.len(),
            });
        }
        Ok(Self::at_incomplete_state(
            graph,
            completed_steps,
            InstallProgressState::Running,
        ))
    }

    pub fn failed(
        graph: &InstallTransactionGraph,
        completed_steps: usize,
    ) -> Result<Self, InstallProgressError> {
        if completed_steps >= graph.steps.len() {
            return Err(InstallProgressError::InvalidCompletedStepCount {
                completed: completed_steps,
                total: graph.steps.len(),
            });
        }
        Ok(Self::at_incomplete_state(
            graph,
            completed_steps,
            InstallProgressState::Failed,
        ))
    }

    pub fn completed(graph: &InstallTransactionGraph) -> Result<Self, InstallProgressError> {
        if graph.steps.is_empty() {
            return Err(InstallProgressError::EmptyTransaction);
        }
        Ok(Self {
            state: InstallProgressState::Completed,
            phase: InstallProgressPhase::Completed,
            operation: "complete",
            completed_steps: graph.steps.len(),
            total_steps: graph.steps.len(),
            percent: 100,
        })
    }

    fn at_incomplete_state(
        graph: &InstallTransactionGraph,
        completed_steps: usize,
        state: InstallProgressState,
    ) -> Self {
        let step = &graph.steps[completed_steps];
        Self {
            state,
            phase: install_progress_phase(step),
            operation: install_progress_operation(step),
            completed_steps,
            total_steps: graph.steps.len(),
            percent: ((completed_steps * 100) / graph.steps.len()).min(99) as u8,
        }
    }

    pub const fn state(&self) -> InstallProgressState {
        self.state
    }

    pub const fn phase(&self) -> InstallProgressPhase {
        self.phase
    }

    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    pub const fn completed_steps(&self) -> usize {
        self.completed_steps
    }

    pub const fn total_steps(&self) -> usize {
        self.total_steps
    }

    pub const fn percent(&self) -> u8 {
        self.percent
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallProgressError {
    EmptyTransaction,
    InvalidCompletedStepCount { completed: usize, total: usize },
}

impl fmt::Display for InstallProgressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTransaction => formatter.write_str("installer transaction has no steps"),
            Self::InvalidCompletedStepCount { completed, total } => write!(
                formatter,
                "invalid completed installer step count {completed}; total is {total}"
            ),
        }
    }
}

impl Error for InstallProgressError {}

fn install_progress_phase(step: &InstallTransactionStep) -> InstallProgressPhase {
    match install_progress_operation(step) {
        "revalidate-target" | "prepare-target-mountpoint" => InstallProgressPhase::PreparingTarget,
        "write-partition-table" => InstallProgressPhase::Partitioning,
        "format-efi-system-partition" | "format-root-partition" => InstallProgressPhase::Formatting,
        "mount-root" | "extract-root-filesystem" | "prepare-efi-mountpoint" => {
            InstallProgressPhase::InstallingSystem
        }
        "mount-efi-system-partition"
        | "prepare-bootloader-directory"
        | "install-kernel"
        | "install-bootloader"
        | "write-bootloader-configuration" => InstallProgressPhase::InstallingBootloader,
        "prepare-system-configuration" | "write-system-configuration" => {
            InstallProgressPhase::ConfiguringSystem
        }
        "unmount-target" => InstallProgressPhase::Finalizing,
        _ => InstallProgressPhase::ConfiguringSystem,
    }
}

fn install_progress_operation(step: &InstallTransactionStep) -> &'static str {
    match step {
        InstallTransactionStep::RevalidateTarget { .. } => "revalidate-target",
        InstallTransactionStep::Command(command) => command.operation(),
        InstallTransactionStep::Internal(action) => match action.kind() {
            InternalInstallActionKind::CreateDirectory { path, .. }
                if path == Path::new(INSTALL_TARGET_ROOT) =>
            {
                "prepare-target-mountpoint"
            }
            InternalInstallActionKind::CreateDirectory { path, .. }
                if path == Path::new("/mnt/aqua-target/boot/efi") =>
            {
                "prepare-efi-mountpoint"
            }
            InternalInstallActionKind::CreateDirectory { path, .. }
                if path == Path::new("/mnt/aqua-target/boot/efi/EFI/BOOT") =>
            {
                "prepare-bootloader-directory"
            }
            InternalInstallActionKind::CreateDirectory { .. } => "prepare-system-configuration",
            InternalInstallActionKind::CopyFileAtomic { destination, .. }
                if destination == Path::new("/mnt/aqua-target/boot/vmlinuz-aqua") =>
            {
                "install-kernel"
            }
            InternalInstallActionKind::CopyFileAtomic { .. } => "install-bootloader",
            InternalInstallActionKind::WriteFileAtomic { destination, .. }
                if destination == Path::new("/mnt/aqua-target/boot/grub/grub.cfg") =>
            {
                "write-bootloader-configuration"
            }
            InternalInstallActionKind::WriteFileAtomic { .. } => "write-system-configuration",
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallTransactionCompileError {
    FingerprintMismatch,
    InvalidPlan(&'static str),
    TooManySteps,
}

impl fmt::Display for InstallTransactionCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FingerprintMismatch => {
                formatter.write_str("installer transaction fingerprints do not match")
            }
            Self::InvalidPlan(reason) => write!(formatter, "invalid transaction plan: {reason}"),
            Self::TooManySteps => formatter.write_str("installer transaction step limit exceeded"),
        }
    }
}

impl Error for InstallTransactionCompileError {}

pub fn build_install_transaction_graph(
    plan: &InstallPlan,
    commands: &InstallCommandPlan,
    internal: &InternalInstallPlan,
) -> Result<InstallTransactionGraph, InstallTransactionCompileError> {
    if plan.operations().len() != 13 {
        return Err(InstallTransactionCompileError::InvalidPlan(
            "canonical source operation count changed",
        ));
    }
    if commands.source_fingerprint() != plan.fingerprint()
        || internal.source_fingerprint() != plan.fingerprint()
    {
        return Err(InstallTransactionCompileError::FingerprintMismatch);
    }
    if commands.commands().len() != 8 || internal.actions().len() != 11 {
        return Err(InstallTransactionCompileError::InvalidPlan(
            "canonical rehearsal counts changed",
        ));
    }
    let InstallPlanOperation::VerifyTarget {
        device,
        stable_id,
        model,
        capacity_bytes,
    } = &plan.operations()[0]
    else {
        return Err(InstallTransactionCompileError::InvalidPlan(
            "target revalidation is not first",
        ));
    };
    let expected = DiskIdentity::new(device, stable_id, model, *capacity_bytes)
        .map_err(|_| InstallTransactionCompileError::InvalidPlan("invalid target identity"))?;
    let expected_command_operations = [
        "write-partition-table",
        "format-efi-system-partition",
        "format-root-partition",
        "mount-root",
        "extract-root-filesystem",
        "mount-efi-system-partition",
        "unmount-target",
        "unmount-target",
    ];
    if !commands
        .commands()
        .iter()
        .zip(expected_command_operations)
        .all(|(command, expected)| command.operation() == expected)
    {
        return Err(InstallTransactionCompileError::InvalidPlan(
            "external command order changed",
        ));
    }
    let anchor_count = |anchor| {
        internal
            .actions()
            .iter()
            .filter(|action| action.anchor() == anchor)
            .count()
    };
    if anchor_count(InternalInstallAnchor::BeforeMountRoot) != 1
        || anchor_count(InternalInstallAnchor::BeforeMountEfi) != 1
        || anchor_count(InternalInstallAnchor::AfterMountEfi) != 9
    {
        return Err(InstallTransactionCompileError::InvalidPlan(
            "internal action anchors changed",
        ));
    }

    let mut steps = vec![InstallTransactionStep::RevalidateTarget { expected }];
    steps.extend(
        internal
            .actions()
            .iter()
            .filter(|action| action.anchor() == InternalInstallAnchor::BeforeMountRoot)
            .cloned()
            .map(InstallTransactionStep::Internal),
    );
    steps.extend(
        commands.commands()[0..=4]
            .iter()
            .cloned()
            .map(InstallTransactionStep::Command),
    );
    steps.extend(
        internal
            .actions()
            .iter()
            .filter(|action| action.anchor() == InternalInstallAnchor::BeforeMountEfi)
            .cloned()
            .map(InstallTransactionStep::Internal),
    );
    steps.push(InstallTransactionStep::Command(
        commands.commands()[5].clone(),
    ));
    steps.extend(
        internal
            .actions()
            .iter()
            .filter(|action| action.anchor() == InternalInstallAnchor::AfterMountEfi)
            .cloned()
            .map(InstallTransactionStep::Internal),
    );
    steps.extend(
        commands.commands()[6..=7]
            .iter()
            .cloned()
            .map(InstallTransactionStep::Command),
    );
    if steps.len() > INSTALL_TRANSACTION_STEP_LIMIT {
        return Err(InstallTransactionCompileError::TooManySteps);
    }
    let cleanup = vec![
        InstallCleanupStep {
            requirement: InstallCleanupRequirement::EfiMounted,
            command: commands.commands()[6].clone(),
        },
        InstallCleanupStep {
            requirement: InstallCleanupRequirement::RootMounted,
            command: commands.commands()[7].clone(),
        },
    ];
    Ok(InstallTransactionGraph {
        source_fingerprint: plan.fingerprint(),
        steps,
        cleanup,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTransactionRehearsalOutcome {
    Completed,
    InjectedFailure { before_step: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTransactionRehearsal {
    source_fingerprint: u64,
    outcome: InstallTransactionRehearsalOutcome,
    rehearsed_steps: Vec<InstallTransactionStep>,
    cleanup: Vec<InstallCleanupStep>,
}

impl InstallTransactionRehearsal {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub const fn outcome(&self) -> InstallTransactionRehearsalOutcome {
        self.outcome
    }

    pub fn rehearsed_steps(&self) -> &[InstallTransactionStep] {
        &self.rehearsed_steps
    }

    pub fn cleanup(&self) -> &[InstallCleanupStep] {
        &self.cleanup
    }

    pub const fn executed(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallTransactionRehearsalError {
    InvalidFailureIndex(usize),
}

impl fmt::Display for InstallTransactionRehearsalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFailureIndex(index) => {
                write!(formatter, "invalid transaction failure index {index}")
            }
        }
    }
}

impl Error for InstallTransactionRehearsalError {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NonExecutingInstallTransactionRunner;

impl NonExecutingInstallTransactionRunner {
    pub fn rehearse(
        &self,
        graph: &InstallTransactionGraph,
        inject_failure_before: Option<usize>,
    ) -> Result<InstallTransactionRehearsal, InstallTransactionRehearsalError> {
        if inject_failure_before.is_some_and(|index| index >= graph.steps.len()) {
            return Err(InstallTransactionRehearsalError::InvalidFailureIndex(
                inject_failure_before.expect("checked failure index"),
            ));
        }
        let mut root_mounted = false;
        let mut efi_mounted = false;
        let mut rehearsed_steps = Vec::with_capacity(graph.steps.len());
        for (index, step) in graph.steps.iter().enumerate() {
            if inject_failure_before == Some(index) {
                let cleanup = graph
                    .cleanup
                    .iter()
                    .filter(|step| match step.requirement {
                        InstallCleanupRequirement::EfiMounted => efi_mounted,
                        InstallCleanupRequirement::RootMounted => root_mounted,
                    })
                    .cloned()
                    .collect();
                return Ok(InstallTransactionRehearsal {
                    source_fingerprint: graph.source_fingerprint,
                    outcome: InstallTransactionRehearsalOutcome::InjectedFailure {
                        before_step: index,
                    },
                    rehearsed_steps,
                    cleanup,
                });
            }
            rehearsed_steps.push(step.clone());
            if let InstallTransactionStep::Command(command) = step {
                match command.operation() {
                    "mount-root" => root_mounted = true,
                    "mount-efi-system-partition" => efi_mounted = true,
                    "unmount-target" => {
                        if command
                            .arguments()
                            .first()
                            .is_some_and(|path| path.ends_with("/boot/efi"))
                        {
                            efi_mounted = false;
                        } else {
                            root_mounted = false;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(InstallTransactionRehearsal {
            source_fingerprint: graph.source_fingerprint,
            outcome: InstallTransactionRehearsalOutcome::Completed,
            rehearsed_steps,
            cleanup: Vec::new(),
        })
    }
}

pub const FIXTURE_INTERNAL_EXECUTOR_STATUS: &str = "root-remapped-fixture-executor-ready";
pub const FIXTURE_COPY_BYTES_LIMIT: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureInstallRoot {
    path: PathBuf,
}

impl FixtureInstallRoot {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, FixtureInstallError> {
        let path = path.into();
        if !path.is_absolute()
            || path == Path::new("/")
            || path == Path::new(INSTALL_TARGET_ROOT)
            || path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(FixtureInstallError::InvalidRoot);
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| fixture_io("inspect fixture root", error))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(FixtureInstallError::InvalidRoot);
        }
        let canonical = fs::canonicalize(&path)
            .map_err(|error| fixture_io("canonicalize fixture root", error))?;
        let canonical_temp = fs::canonicalize(std::env::temp_dir())
            .map_err(|error| fixture_io("canonicalize temporary root", error))?;
        if canonical == canonical_temp || !canonical.starts_with(&canonical_temp) {
            return Err(FixtureInstallError::OutsideTemporaryRoot);
        }
        if fs::read_dir(&canonical)
            .map_err(|error| fixture_io("read fixture root", error))?
            .next()
            .is_some()
        {
            return Err(FixtureInstallError::RootNotEmpty);
        }
        Ok(Self { path: canonical })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn remap(&self, target: &Path) -> Result<PathBuf, FixtureInstallError> {
        validate_internal_destination(target, "fixture-target")
            .map_err(|_| FixtureInstallError::UnsafeTarget(target.to_path_buf()))?;
        let relative = target
            .strip_prefix(INSTALL_TARGET_ROOT)
            .map_err(|_| FixtureInstallError::UnsafeTarget(target.to_path_buf()))?;
        Ok(self.path.join(relative))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureInstallError {
    InvalidRoot,
    OutsideTemporaryRoot,
    RootNotEmpty,
    UnsafeTarget(PathBuf),
    SymlinkComponent(PathBuf),
    InvalidSource(PathBuf),
    SourceTooLarge {
        path: PathBuf,
        size: u64,
    },
    TemporaryExists(PathBuf),
    Io {
        context: &'static str,
        kind: io::ErrorKind,
        message: String,
    },
}

impl fmt::Display for FixtureInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot => formatter.write_str("invalid fixture root"),
            Self::OutsideTemporaryRoot => {
                formatter.write_str("fixture root is outside the system temporary directory")
            }
            Self::RootNotEmpty => formatter.write_str("fixture root must start empty"),
            Self::UnsafeTarget(path) => {
                write!(formatter, "unsafe fixture target {}", path.display())
            }
            Self::SymlinkComponent(path) => {
                write!(
                    formatter,
                    "fixture path contains symlink {}",
                    path.display()
                )
            }
            Self::InvalidSource(path) => {
                write!(formatter, "invalid fixture source {}", path.display())
            }
            Self::SourceTooLarge { path, size } => {
                write!(
                    formatter,
                    "fixture source {} is too large: {size}",
                    path.display()
                )
            }
            Self::TemporaryExists(path) => {
                write!(
                    formatter,
                    "fixture temporary path exists: {}",
                    path.display()
                )
            }
            Self::Io {
                context, message, ..
            } => write!(formatter, "{context}: {message}"),
        }
    }
}

impl Error for FixtureInstallError {}

fn fixture_io(context: &'static str, error: io::Error) -> FixtureInstallError {
    FixtureInstallError::Io {
        context,
        kind: error.kind(),
        message: error.to_string(),
    }
}

fn fixture_ensure_directory(
    root: &FixtureInstallRoot,
    directory: &Path,
    mode: u32,
) -> Result<(), FixtureInstallError> {
    if !directory.starts_with(root.path()) {
        return Err(FixtureInstallError::UnsafeTarget(directory.to_path_buf()));
    }
    let relative = directory
        .strip_prefix(root.path())
        .map_err(|_| FixtureInstallError::UnsafeTarget(directory.to_path_buf()))?;
    let mut current = root.path().to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(FixtureInstallError::UnsafeTarget(directory.to_path_buf()));
        };
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(FixtureInstallError::SymlinkComponent(current));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(FixtureInstallError::UnsafeTarget(current));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&current)
                    .map_err(|error| fixture_io("create fixture directory", error))?;
            }
            Err(error) => return Err(fixture_io("inspect fixture directory", error)),
        }
    }
    let mut permissions = fs::metadata(directory)
        .map_err(|error| fixture_io("read fixture directory mode", error))?
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(directory, permissions)
        .map_err(|error| fixture_io("set fixture directory mode", error))
}

fn fixture_validate_file_target(
    root: &FixtureInstallRoot,
    destination: &Path,
    temporary: &Path,
) -> Result<(), FixtureInstallError> {
    if !destination.starts_with(root.path())
        || !temporary.starts_with(root.path())
        || destination.parent() != temporary.parent()
        || destination == temporary
    {
        return Err(FixtureInstallError::UnsafeTarget(destination.to_path_buf()));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| FixtureInstallError::UnsafeTarget(destination.to_path_buf()))?;
    fixture_ensure_directory(root, parent, 0o755)?;
    for path in [destination, temporary] {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(FixtureInstallError::SymlinkComponent(path.to_path_buf()));
            }
            Ok(_) if path == temporary => {
                return Err(FixtureInstallError::TemporaryExists(path.to_path_buf()));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(FixtureInstallError::UnsafeTarget(path.to_path_buf()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(fixture_io("inspect fixture file target", error)),
        }
    }
    Ok(())
}

fn fixture_atomic_write(
    root: &FixtureInstallRoot,
    destination: &Path,
    temporary: &Path,
    content: &[u8],
    mode: u32,
) -> Result<(), FixtureInstallError> {
    fixture_validate_file_target(root, destination, temporary)?;
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temporary)
            .map_err(|error| fixture_io("create fixture temporary file", error))?;
        file.write_all(content)
            .map_err(|error| fixture_io("write fixture temporary file", error))?;
        let mut permissions = file
            .metadata()
            .map_err(|error| fixture_io("read fixture temporary mode", error))?
            .permissions();
        permissions.set_mode(mode);
        file.set_permissions(permissions)
            .map_err(|error| fixture_io("set fixture temporary mode", error))?;
        file.sync_all()
            .map_err(|error| fixture_io("sync fixture temporary file", error))?;
        drop(file);
        fs::rename(temporary, destination)
            .map_err(|error| fixture_io("commit fixture atomic file", error))
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FixtureInternalInstallExecutor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureInstallReport {
    source_fingerprint: u64,
    action_count: usize,
}

impl FixtureInstallReport {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub const fn action_count(&self) -> usize {
        self.action_count
    }

    pub const fn fixture_filesystem_executed(&self) -> bool {
        true
    }

    pub const fn disk_commands_executed(&self) -> bool {
        false
    }
}

impl FixtureInternalInstallExecutor {
    pub fn execute(
        &self,
        root: &FixtureInstallRoot,
        plan: &InternalInstallPlan,
    ) -> Result<FixtureInstallReport, FixtureInstallError> {
        for action in plan.actions() {
            match action.kind() {
                InternalInstallActionKind::CreateDirectory { path, mode } => {
                    let mapped = root.remap(path)?;
                    fixture_ensure_directory(root, &mapped, *mode)?;
                }
                InternalInstallActionKind::CopyFileAtomic {
                    source,
                    destination,
                    temporary,
                    mode,
                } => {
                    let source_metadata = fs::symlink_metadata(source)
                        .map_err(|_| FixtureInstallError::InvalidSource(source.clone()))?;
                    if source_metadata.file_type().is_symlink() || !source_metadata.is_file() {
                        return Err(FixtureInstallError::InvalidSource(source.clone()));
                    }
                    if source_metadata.len() > FIXTURE_COPY_BYTES_LIMIT {
                        return Err(FixtureInstallError::SourceTooLarge {
                            path: source.clone(),
                            size: source_metadata.len(),
                        });
                    }
                    let content = fs::read(source)
                        .map_err(|error| fixture_io("read fixture copy source", error))?;
                    fixture_atomic_write(
                        root,
                        &root.remap(destination)?,
                        &root.remap(temporary)?,
                        &content,
                        *mode,
                    )?;
                }
                InternalInstallActionKind::WriteFileAtomic {
                    destination,
                    temporary,
                    content,
                    mode,
                } => fixture_atomic_write(
                    root,
                    &root.remap(destination)?,
                    &root.remap(temporary)?,
                    content.as_bytes(),
                    *mode,
                )?,
            }
        }
        Ok(FixtureInstallReport {
            source_fingerprint: plan.source_fingerprint(),
            action_count: plan.actions().len(),
        })
    }
}

pub const FIXTURE_TOOL_SHIM_RUNNER_STATUS: &str = "temporary-tool-shim-runner-ready";
pub const FIXTURE_TOOL_SHIM_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureToolShimRoot {
    path: PathBuf,
    log_directory: PathBuf,
}

impl FixtureToolShimRoot {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, FixtureToolShimError> {
        let path = path.into();
        if !path.is_absolute()
            || path == Path::new("/")
            || path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(FixtureToolShimError::InvalidRoot);
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| fixture_tool_io("inspect shim root", error))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(FixtureToolShimError::InvalidRoot);
        }
        let canonical = fs::canonicalize(&path)
            .map_err(|error| fixture_tool_io("canonicalize shim root", error))?;
        let canonical_temp = fs::canonicalize(std::env::temp_dir())
            .map_err(|error| fixture_tool_io("canonicalize temporary root", error))?;
        if canonical == canonical_temp || !canonical.starts_with(&canonical_temp) {
            return Err(FixtureToolShimError::OutsideTemporaryRoot);
        }
        let log_directory = path.join("logs");
        match fs::symlink_metadata(&log_directory) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(FixtureToolShimError::InvalidRoot);
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&log_directory)
                    .map_err(|error| fixture_tool_io("create shim log directory", error))?;
            }
            Err(error) => return Err(fixture_tool_io("inspect shim log directory", error)),
        }
        Ok(Self {
            path,
            log_directory,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn log_directory(&self) -> &Path {
        &self.log_directory
    }

    fn validate_program(&self, program: &Path) -> Result<(), FixtureToolShimError> {
        if !program.is_absolute() || !program.starts_with(&self.path) {
            return Err(FixtureToolShimError::ProgramOutsideRoot(
                program.to_path_buf(),
            ));
        }
        let relative = program
            .strip_prefix(&self.path)
            .map_err(|_| FixtureToolShimError::ProgramOutsideRoot(program.to_path_buf()))?;
        let mut current = self.path.clone();
        for component in relative.components() {
            let Component::Normal(component) = component else {
                return Err(FixtureToolShimError::InvalidProgram(program.to_path_buf()));
            };
            current.push(component);
            let metadata = fs::symlink_metadata(&current)
                .map_err(|_| FixtureToolShimError::InvalidProgram(program.to_path_buf()))?;
            if metadata.file_type().is_symlink() {
                return Err(FixtureToolShimError::SymlinkComponent(current));
            }
        }
        let metadata = fs::metadata(program)
            .map_err(|_| FixtureToolShimError::InvalidProgram(program.to_path_buf()))?;
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            return Err(FixtureToolShimError::InvalidProgram(program.to_path_buf()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureToolShimError {
    InvalidRoot,
    OutsideTemporaryRoot,
    ProgramOutsideRoot(PathBuf),
    InvalidProgram(PathBuf),
    SymlinkComponent(PathBuf),
    Spawn {
        operation: &'static str,
        kind: io::ErrorKind,
        message: String,
    },
    Stdin {
        operation: &'static str,
        kind: io::ErrorKind,
        message: String,
    },
    Wait {
        operation: &'static str,
        kind: io::ErrorKind,
        message: String,
    },
    InvalidFailureStep(usize),
    Io {
        context: &'static str,
        kind: io::ErrorKind,
        message: String,
    },
}

impl fmt::Display for FixtureToolShimError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot => formatter.write_str("invalid fixture tool-shim root"),
            Self::OutsideTemporaryRoot => {
                formatter.write_str("tool-shim root is outside the system temporary directory")
            }
            Self::ProgramOutsideRoot(path) => {
                write!(
                    formatter,
                    "shim program is outside root: {}",
                    path.display()
                )
            }
            Self::InvalidProgram(path) => {
                write!(formatter, "invalid shim program: {}", path.display())
            }
            Self::SymlinkComponent(path) => {
                write!(formatter, "shim path contains symlink: {}", path.display())
            }
            Self::Spawn {
                operation, message, ..
            } => {
                write!(formatter, "spawn shim for {operation}: {message}")
            }
            Self::Stdin {
                operation, message, ..
            } => {
                write!(formatter, "write shim stdin for {operation}: {message}")
            }
            Self::Wait {
                operation, message, ..
            } => {
                write!(formatter, "wait for shim {operation}: {message}")
            }
            Self::InvalidFailureStep(index) => {
                write!(
                    formatter,
                    "cannot map shim failure at transaction step {index}"
                )
            }
            Self::Io {
                context, message, ..
            } => write!(formatter, "{context}: {message}"),
        }
    }
}

impl Error for FixtureToolShimError {}

fn fixture_tool_io(context: &'static str, error: io::Error) -> FixtureToolShimError {
    FixtureToolShimError::Io {
        context,
        kind: error.kind(),
        message: error.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureToolShimOutcome {
    Completed,
    Failed {
        transaction_step: usize,
        operation: &'static str,
        exit_code: Option<i32>,
        timed_out: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureToolShimReport {
    source_fingerprint: u64,
    outcome: FixtureToolShimOutcome,
    completed_commands: Vec<InstallCommandSpec>,
    cleanup: Vec<InstallCleanupStep>,
}

impl FixtureToolShimReport {
    pub const fn source_fingerprint(&self) -> u64 {
        self.source_fingerprint
    }

    pub const fn outcome(&self) -> &FixtureToolShimOutcome {
        &self.outcome
    }

    pub fn completed_commands(&self) -> &[InstallCommandSpec] {
        &self.completed_commands
    }

    pub fn cleanup(&self) -> &[InstallCleanupStep] {
        &self.cleanup
    }

    pub const fn shim_processes_executed(&self) -> bool {
        true
    }

    pub const fn real_disk_tools_executed(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FixtureToolShimRunner;

impl FixtureToolShimRunner {
    pub fn execute_transaction_commands(
        &self,
        root: &FixtureToolShimRoot,
        graph: &InstallTransactionGraph,
    ) -> Result<FixtureToolShimReport, FixtureToolShimError> {
        let mut completed_commands = Vec::new();
        for (transaction_step, step) in graph.steps().iter().enumerate() {
            let InstallTransactionStep::Command(command) = step else {
                continue;
            };
            root.validate_program(command.program())?;
            let stdin_mode = if command.stdin().is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            };
            let mut child = Command::new(command.program())
                .args(command.arguments())
                .current_dir(root.path())
                .env_clear()
                .env("AQUA_FIXTURE_LOG_DIR", root.log_directory())
                .stdin(stdin_mode)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|error| FixtureToolShimError::Spawn {
                    operation: command.operation(),
                    kind: error.kind(),
                    message: error.to_string(),
                })?;
            if let Some(payload) = command.stdin() {
                child
                    .stdin
                    .take()
                    .ok_or(FixtureToolShimError::Stdin {
                        operation: command.operation(),
                        kind: io::ErrorKind::BrokenPipe,
                        message: "shim stdin pipe unavailable".to_string(),
                    })?
                    .write_all(payload.as_bytes())
                    .map_err(|error| FixtureToolShimError::Stdin {
                        operation: command.operation(),
                        kind: error.kind(),
                        message: error.to_string(),
                    })?;
            }
            let deadline = Instant::now() + FIXTURE_TOOL_SHIM_TIMEOUT;
            let (status, timed_out) = loop {
                match child
                    .try_wait()
                    .map_err(|error| FixtureToolShimError::Wait {
                        operation: command.operation(),
                        kind: error.kind(),
                        message: error.to_string(),
                    })? {
                    Some(status) => break (status, false),
                    None if Instant::now() < deadline => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    None => {
                        child.kill().map_err(|error| FixtureToolShimError::Wait {
                            operation: command.operation(),
                            kind: error.kind(),
                            message: format!("kill timed-out shim: {error}"),
                        })?;
                        let status = child.wait().map_err(|error| FixtureToolShimError::Wait {
                            operation: command.operation(),
                            kind: error.kind(),
                            message: format!("reap timed-out shim: {error}"),
                        })?;
                        break (status, true);
                    }
                }
            };
            if !status.success() || timed_out {
                let rehearsal = NonExecutingInstallTransactionRunner
                    .rehearse(graph, Some(transaction_step))
                    .map_err(|_| FixtureToolShimError::InvalidFailureStep(transaction_step))?;
                return Ok(FixtureToolShimReport {
                    source_fingerprint: graph.source_fingerprint(),
                    outcome: FixtureToolShimOutcome::Failed {
                        transaction_step,
                        operation: command.operation(),
                        exit_code: status.code(),
                        timed_out,
                    },
                    completed_commands,
                    cleanup: rehearsal.cleanup().to_vec(),
                });
            }
            completed_commands.push(command.clone());
        }
        Ok(FixtureToolShimReport {
            source_fingerprint: graph.source_fingerprint(),
            outcome: FixtureToolShimOutcome::Completed,
            completed_commands,
            cleanup: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_ROOT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestStorageRoot {
        path: PathBuf,
    }

    impl TestStorageRoot {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let sequence = TEST_ROOT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "aqua-installer-storage-{}-{nonce}-{sequence}",
                std::process::id(),
            ));
            fs::create_dir_all(path.join("sys/class/block")).unwrap();
            fs::create_dir_all(path.join("devices")).unwrap();
            fs::create_dir_all(path.join("proc/self")).unwrap();
            fs::write(path.join("proc/cmdline"), "console=ttyS0\n").unwrap();
            fs::write(
                path.join("proc/self/mountinfo"),
                "20 1 0:1 / / rw - rootfs rootfs rw\n",
            )
            .unwrap();
            Self { path }
        }

        fn paths(&self) -> StorageProbePaths {
            StorageProbePaths {
                sys_class_block: self.path.join("sys/class/block"),
                proc_mountinfo: self.path.join("proc/self/mountinfo"),
                proc_cmdline: self.path.join("proc/cmdline"),
            }
        }

        fn add_disk(&self, name: &str, major_minor: &str, sectors: u64, read_only: bool) {
            let device = self.path.join("devices").join(name);
            fs::create_dir_all(device.join("device")).unwrap();
            fs::write(device.join("size"), format!("{sectors}\n")).unwrap();
            fs::write(device.join("dev"), format!("{major_minor}\n")).unwrap();
            fs::write(device.join("ro"), if read_only { "1\n" } else { "0\n" }).unwrap();
            fs::write(device.join("removable"), "0\n").unwrap();
            fs::write(device.join("uevent"), "DEVTYPE=disk\n").unwrap();
            fs::write(device.join("device/model"), "QEMU HARDDISK\n").unwrap();
            fs::write(device.join("device/serial"), format!("aqua-{name}\n")).unwrap();
            symlink(device, self.path.join("sys/class/block").join(name)).unwrap();
        }

        fn add_partition(&self, disk: &str, name: &str, major_minor: &str) {
            let partition = self.path.join("devices").join(disk).join(name);
            fs::create_dir_all(&partition).unwrap();
            fs::write(partition.join("partition"), "1\n").unwrap();
            fs::write(partition.join("dev"), format!("{major_minor}\n")).unwrap();
            fs::write(partition.join("uevent"), "DEVTYPE=partition\n").unwrap();
            symlink(partition, self.path.join("sys/class/block").join(name)).unwrap();
        }
    }

    impl Drop for TestStorageRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn disk(device: &str, stable_id: &str) -> DiskIdentity {
        DiskIdentity::new(device, stable_id, "QEMU HARDDISK", 32 * 1024 * 1024 * 1024)
            .expect("valid test disk")
    }

    fn ready_model(mode: InstallMode) -> InstallerModel {
        let mut model = InstallerModel::default();
        model.set_mode(mode);
        assert_eq!(model.advance(), Ok(InstallerStep::Language));
        model.set_locale("tr_TR.UTF-8").unwrap();
        assert_eq!(model.advance(), Ok(InstallerStep::Keyboard));
        model.set_keyboard_layout("tr").unwrap();
        assert_eq!(model.advance(), Ok(InstallerStep::Partitions));
        model.set_target(InstallTarget::erase_disk(disk(
            "/dev/vda",
            "virtio-aqua-target",
        )));
        assert_eq!(model.advance(), Ok(InstallerStep::TimeZone));
        model.set_timezone("Europe/Istanbul").unwrap();
        assert_eq!(model.advance(), Ok(InstallerStep::UserInformation));
        model.set_user(UserProfile::new("aqua", "Aqua User", true).unwrap());
        assert_eq!(model.advance(), Ok(InstallerStep::Summary));
        model
    }

    #[test]
    fn canonical_step_order_matches_ui_contract() {
        assert_eq!(
            InstallerStep::ALL.map(InstallerStep::id),
            [
                "welcome",
                "language",
                "keyboard",
                "partitions",
                "time-zone",
                "user-information",
                "summary",
                "installation",
                "completed",
            ]
        );
        assert_eq!(
            InstallerStep::ALL.map(InstallerStep::label_tr),
            [
                "Hoş Geldiniz",
                "Dil",
                "Klavye",
                "Bölümler",
                "Zaman Dilimi",
                "Kullanıcı Bilgisi",
                "Özet",
                "Kurulum",
                "Tamamlandı",
            ]
        );
    }

    #[test]
    fn installer_window_layout_fits_supported_viewports() {
        for viewport in [
            Viewport::new(800, 600),
            Viewport::new(1280, 800),
            Viewport::new(1536, 1024),
        ] {
            let layout = InstallerWindowLayout::for_viewport(viewport).unwrap();
            assert!(layout.fits_viewport());
            assert!(layout.regions_are_separated());
            assert_eq!(layout.surfaces().len(), 6);
            assert!(layout.progress_track.width > 0);
        }

        let canonical = InstallerWindowLayout::for_viewport(Viewport::new(1536, 1024)).unwrap();
        assert_eq!(canonical.window.x, 32);
        assert_eq!(canonical.window.y, 32);
        assert_eq!(canonical.window.width, 1472);
        assert_eq!(canonical.window.height, 960);
        assert_eq!(canonical.step_rail.width, 264);
        assert_eq!(
            InstallerWindowLayout::for_viewport(Viewport::new(799, 600)),
            Err(InstallerUiLayoutError::UnsupportedViewport(Viewport::new(
                799, 600
            )))
        );
    }

    #[test]
    fn installer_keyboard_focus_wraps_and_tracks_terminal_steps() {
        let mut model = InstallerModel::default();
        let mut ui = InstallerUiState::new(&model);
        assert_eq!(ui.focus(), InstallerFocusTarget::LanguageControl);
        assert_eq!(
            ui.handle_key(InstallerUiKey::Tab),
            InstallerUiAction::FocusChanged(InstallerFocusTarget::Cancel)
        );
        assert_eq!(
            ui.handle_key(InstallerUiKey::End),
            InstallerUiAction::FocusChanged(InstallerFocusTarget::Forward)
        );
        assert_eq!(
            ui.handle_key(InstallerUiKey::Tab),
            InstallerUiAction::FocusChanged(InstallerFocusTarget::LanguageControl)
        );
        assert_eq!(
            ui.handle_key(InstallerUiKey::Escape),
            InstallerUiAction::CancelRequested
        );

        model.advance().unwrap();
        assert!(ui.sync_step(&model));
        assert_eq!(ui.focus(), InstallerFocusTarget::StepContent);
        assert_eq!(
            ui.handle_key(InstallerUiKey::BackTab),
            InstallerUiAction::FocusChanged(InstallerFocusTarget::Forward)
        );
        assert_eq!(
            ui.handle_key(InstallerUiKey::Activate),
            InstallerUiAction::AdvanceRequested
        );

        let mut model = ready_model(InstallMode::DryRun);
        let mut ui = InstallerUiState::new(&model);
        assert_eq!(ui.step(), InstallerStep::Summary);
        assert_eq!(ui.forward_label(), Some("Kur"));
        ui.handle_key(InstallerUiKey::End);
        assert_eq!(
            ui.handle_key(InstallerUiKey::Activate),
            InstallerUiAction::BeginInstallRequested
        );

        model.begin_install().unwrap();
        assert!(ui.sync_step(&model));
        assert_eq!(ui.focus(), InstallerFocusTarget::ProgressStatus);
        assert_eq!(ui.forward_label(), None);
        assert!(!ui.back_visible());
        assert!(!ui.cancel_visible());
        assert_eq!(
            ui.handle_key(InstallerUiKey::Escape),
            InstallerUiAction::None
        );

        model.complete_install().unwrap();
        assert!(ui.sync_step(&model));
        assert_eq!(ui.focus(), InstallerFocusTarget::Finish);
        assert_eq!(ui.forward_label(), Some("Yeniden Başlat"));
        assert_eq!(
            ui.handle_key(InstallerUiKey::Activate),
            InstallerUiAction::FinishRequested
        );
    }

    #[test]
    fn installer_pointer_footer_uses_step_specific_actions() {
        let layout = InstallerWindowLayout::for_viewport(Viewport::new(1280, 800)).unwrap();
        let center = |rect: Rect| (rect.x + rect.width / 2, rect.y + rect.height / 2);

        let mut model = InstallerModel::default();
        let mut ui = InstallerUiState::new(&model);
        let (x, y) = center(layout.forward_button);
        assert_eq!(
            ui.handle_pointer(&layout, x, y),
            InstallerUiAction::AdvanceRequested
        );
        assert_eq!(ui.focus(), InstallerFocusTarget::Forward);
        let (x, y) = center(layout.back_button);
        assert_eq!(ui.handle_pointer(&layout, x, y), InstallerUiAction::None);

        model = ready_model(InstallMode::DryRun);
        ui = InstallerUiState::new(&model);
        let (x, y) = center(layout.forward_button);
        assert_eq!(
            ui.handle_pointer(&layout, x, y),
            InstallerUiAction::BeginInstallRequested
        );

        model.begin_install().unwrap();
        ui.sync_step(&model);
        assert_eq!(ui.handle_pointer(&layout, x, y), InstallerUiAction::None);

        model.complete_install().unwrap();
        ui.sync_step(&model);
        assert_eq!(
            ui.handle_pointer(&layout, x, y),
            InstallerUiAction::FinishRequested
        );
        assert_eq!(ui.focus(), InstallerFocusTarget::Finish);

        let (x, y) = center(layout.language_control);
        assert_eq!(
            ui.handle_pointer(&layout, x, y),
            InstallerUiAction::OpenLanguageControl
        );
        assert_eq!(ui.handle_pointer(&layout, 0, 0), InstallerUiAction::None);
    }

    #[test]
    fn installer_pointer_content_selects_without_implicit_disk_or_user_activation() {
        let layout = InstallerWindowLayout::for_viewport(Viewport::new(1280, 800)).unwrap();
        let center = |rect: Rect| (rect.x + rect.width / 2, rect.y + rect.height / 2);
        let mut model = InstallerModel::default();
        let mut forms = InstallerFormState::default();
        let mut ui = InstallerUiState::new(&model);

        assert_eq!(ui.focus_step_content(), InstallerUiAction::None);
        model.advance().unwrap();
        ui.sync_step(&model);
        let (x, y) = center(layout.choice_row(1));
        assert_eq!(
            forms
                .handle_choice_pointer(&mut model, &layout, x, y)
                .unwrap(),
            InstallerFormUpdate::SelectionChanged {
                step: InstallerStep::Language,
                index: 1,
                value: "en_US.UTF-8",
            }
        );
        assert_eq!(model.locale(), Some("en_US.UTF-8"));
        assert_eq!(
            ui.focus_step_content(),
            InstallerUiAction::FocusChanged(InstallerFocusTarget::StepContent)
        );

        model.advance().unwrap();
        let (x, y) = center(layout.choice_row(2));
        forms
            .handle_choice_pointer(&mut model, &layout, x, y)
            .unwrap();
        assert_eq!(model.keyboard_layout(), Some("us"));
        model.advance().unwrap();

        let root = TestStorageRoot::new();
        root.add_disk("vda", "252:0", 67_108_864, true);
        root.add_disk("vdb", "252:16", 67_108_864, false);
        root.add_disk("vdc", "252:32", 134_217_728, false);
        forms.load_storage_inventory(&probe_storage(&root.paths()).unwrap());
        let (x, y) = center(layout.disk_row(0));
        assert_eq!(
            forms.handle_disk_pointer(&model, &layout, x, y),
            InstallerDiskFormUpdate::None
        );
        let (x, y) = center(layout.disk_row(2));
        assert_eq!(
            forms.handle_disk_pointer(&model, &layout, x, y),
            InstallerDiskFormUpdate::SelectionChanged { index: 2 }
        );
        assert!(model.target().is_none());
        forms
            .handle_disk_key(&mut model, InstallerFormKey::Activate)
            .unwrap();

        model.advance().unwrap();
        model.set_timezone("Europe/Istanbul").unwrap();
        model.advance().unwrap();
        let (x, y) = center(layout.user_field_row(InstallerUserField::DisplayName));
        assert_eq!(
            forms.user_mut().handle_pointer(&model, &layout, x, y),
            InstallerUserFormUpdate::FieldChanged(InstallerUserField::DisplayName)
        );
        assert_eq!(forms.user().active_field(), InstallerUserField::DisplayName);
        assert!(model.user().is_none());
    }

    #[test]
    fn language_and_keyboard_forms_apply_bounded_catalog_values() {
        let mut model = InstallerModel::default();
        model.advance().unwrap();
        let mut forms = InstallerFormState::default();

        assert_eq!(forms.selected_index(model.step()), Some(0));
        assert_eq!(
            forms
                .handle_key(&mut model, InstallerFormKey::Down)
                .unwrap(),
            InstallerFormUpdate::SelectionChanged {
                step: InstallerStep::Language,
                index: 1,
                value: "en_US.UTF-8",
            }
        );
        assert_eq!(model.locale(), Some("en_US.UTF-8"));
        assert_eq!(
            forms.handle_key(&mut model, InstallerFormKey::End).unwrap(),
            InstallerFormUpdate::SelectionChanged {
                step: InstallerStep::Language,
                index: 2,
                value: "de_DE.UTF-8",
            }
        );
        assert_eq!(
            forms
                .handle_key(&mut model, InstallerFormKey::Activate)
                .unwrap(),
            InstallerFormUpdate::ValueApplied {
                step: InstallerStep::Language,
                index: 2,
                value: "de_DE.UTF-8",
            }
        );

        model.advance().unwrap();
        assert_eq!(model.step(), InstallerStep::Keyboard);
        assert_eq!(
            forms.handle_key(&mut model, InstallerFormKey::Up).unwrap(),
            InstallerFormUpdate::SelectionChanged {
                step: InstallerStep::Keyboard,
                index: 2,
                value: "us",
            }
        );
        assert_eq!(model.keyboard_layout(), Some("us"));
        assert_eq!(
            forms
                .handle_key(&mut model, InstallerFormKey::Home)
                .unwrap(),
            InstallerFormUpdate::SelectionChanged {
                step: InstallerStep::Keyboard,
                index: 0,
                value: "trq",
            }
        );

        let mut restored = InstallerFormState::default();
        restored.sync_model(&model);
        assert_eq!(restored.language_index(), 2);
        assert_eq!(restored.keyboard_index(), 0);
        assert!(
            forms
                .handle_key(&mut InstallerModel::default(), InstallerFormKey::Down)
                .unwrap()
                == InstallerFormUpdate::None
        );
    }

    #[test]
    fn disk_form_skips_blocked_storage_and_applies_only_on_activation() {
        let root = TestStorageRoot::new();
        root.add_disk("vda", "252:0", 67_108_864, true);
        root.add_disk("vdb", "252:16", 67_108_864, false);
        root.add_disk("vdc", "252:32", 134_217_728, false);
        let inventory = probe_storage(&root.paths()).unwrap();
        let mut forms = InstallerFormState::default();
        forms.load_storage_inventory(&inventory);
        assert_eq!(forms.disk_options().len(), 3);
        assert_eq!(forms.disk_index(), Some(1));
        assert!(!forms.disk_options()[0].is_eligible());

        let mut model = InstallerModel::default();
        model.advance().unwrap();
        model.set_locale("tr_TR.UTF-8").unwrap();
        model.advance().unwrap();
        model.set_keyboard_layout("trq").unwrap();
        model.advance().unwrap();
        assert_eq!(model.step(), InstallerStep::Partitions);

        assert_eq!(
            forms
                .handle_disk_key(&mut model, InstallerFormKey::Down)
                .unwrap(),
            InstallerDiskFormUpdate::SelectionChanged { index: 2 }
        );
        assert!(model.target().is_none());
        assert_eq!(
            forms
                .handle_disk_key(&mut model, InstallerFormKey::Activate)
                .unwrap(),
            InstallerDiskFormUpdate::TargetApplied { index: 2 }
        );
        assert_eq!(model.target().unwrap().disk.device(), "/dev/vdc");

        assert_eq!(
            forms
                .handle_disk_key(&mut model, InstallerFormKey::Up)
                .unwrap(),
            InstallerDiskFormUpdate::SelectionChanged { index: 1 }
        );
        assert_eq!(
            forms.disk_options()[forms.disk_index().unwrap()].device(),
            "/dev/vdb"
        );
    }

    #[test]
    fn timezone_form_applies_only_bounded_iana_catalog_values() {
        let mut model = InstallerModel::default();
        model.advance().unwrap();
        model.set_locale("tr_TR.UTF-8").unwrap();
        model.advance().unwrap();
        model.set_keyboard_layout("trq").unwrap();
        model.advance().unwrap();
        model.set_target(InstallTarget::erase_disk(disk(
            "/dev/vdb",
            "timezone-form-target",
        )));
        model.advance().unwrap();
        let mut forms = InstallerFormState::default();

        assert_eq!(forms.selected_index(model.step()), Some(0));
        assert_eq!(
            forms.handle_key(&mut model, InstallerFormKey::End).unwrap(),
            InstallerFormUpdate::SelectionChanged {
                step: InstallerStep::TimeZone,
                index: 3,
                value: "America/New_York",
            }
        );
        assert_eq!(model.timezone(), Some("America/New_York"));
        assert_eq!(
            forms
                .handle_key(&mut model, InstallerFormKey::Activate)
                .unwrap(),
            InstallerFormUpdate::ValueApplied {
                step: InstallerStep::TimeZone,
                index: 3,
                value: "America/New_York",
            }
        );

        let mut restored = InstallerFormState::default();
        restored.sync_model(&model);
        assert_eq!(restored.timezone_index(), 3);
    }

    #[test]
    fn user_form_applies_valid_profile_without_accepting_password_characters() {
        let mut model = InstallerModel::default();
        model.advance().unwrap();
        model.set_locale("tr_TR.UTF-8").unwrap();
        model.advance().unwrap();
        model.set_keyboard_layout("trq").unwrap();
        model.advance().unwrap();
        model.set_target(InstallTarget::erase_disk(disk(
            "/dev/vdb",
            "user-form-target",
        )));
        model.advance().unwrap();
        model.set_timezone("Europe/Istanbul").unwrap();
        model.advance().unwrap();
        let mut form = InstallerUserFormState::default();

        for character in "aqua_1".chars() {
            assert!(form
                .handle_key(&mut model, InstallerUserFormKey::Character(character))
                .unwrap()
                .changed());
        }
        assert_eq!(form.username(), "aqua_1");
        assert_eq!(
            form.handle_key(&mut model, InstallerUserFormKey::NextField)
                .unwrap(),
            InstallerUserFormUpdate::FieldChanged(InstallerUserField::DisplayName)
        );
        for character in "Aqua User".chars() {
            form.handle_key(&mut model, InstallerUserFormKey::Character(character))
                .unwrap();
        }
        form.handle_key(&mut model, InstallerUserFormKey::NextField)
            .unwrap();
        assert_eq!(form.active_field(), InstallerUserField::Password);
        assert_eq!(
            form.handle_key(&mut model, InstallerUserFormKey::Character('x'))
                .unwrap(),
            InstallerUserFormUpdate::None
        );
        assert_eq!(
            form.handle_key(
                &mut model,
                InstallerUserFormKey::SetPasswordConfigured(true),
            )
            .unwrap(),
            InstallerUserFormUpdate::PasswordStatusChanged(true)
        );
        assert_eq!(
            form.handle_key(&mut model, InstallerUserFormKey::Activate)
                .unwrap(),
            InstallerUserFormUpdate::ProfileApplied
        );
        let user = model.user().unwrap();
        assert_eq!(user.username(), "aqua_1");
        assert_eq!(user.display_name(), "Aqua User");
        assert!(user.password_configured());
    }

    #[test]
    fn summary_form_requires_exact_target_bound_confirmation_in_real_mode() {
        let mut model = ready_model(InstallMode::Real);
        let mut summary = InstallerSummaryState::default();
        for character in "ERASE /dev/vdc".chars() {
            summary
                .handle_key(&mut model, InstallerSummaryKey::Character(character))
                .unwrap();
        }
        assert_eq!(
            summary.handle_key(&mut model, InstallerSummaryKey::Activate),
            Err(InstallerError::ConfirmationPhraseMismatch)
        );
        assert!(!model.destructive_confirmed());
        summary
            .handle_key(&mut model, InstallerSummaryKey::Clear)
            .unwrap();
        for character in model.confirmation_phrase().unwrap().chars() {
            summary
                .handle_key(&mut model, InstallerSummaryKey::Character(character))
                .unwrap();
        }
        assert_eq!(
            summary
                .handle_key(&mut model, InstallerSummaryKey::Activate)
                .unwrap(),
            InstallerSummaryUpdate::ConfirmationApplied
        );
        assert!(summary.can_begin_install(&model));

        model.set_target(InstallTarget::erase_disk(disk(
            "/dev/vdc",
            "summary-target-changed",
        )));
        assert!(!model.destructive_confirmed());
        assert!(!summary.can_begin_install(&model));

        let mut dry_run = ready_model(InstallMode::DryRun);
        assert_eq!(
            InstallerSummaryState::default()
                .handle_key(&mut dry_run, InstallerSummaryKey::Activate)
                .unwrap(),
            InstallerSummaryUpdate::ReadyToInstall
        );
    }

    #[test]
    fn required_selection_blocks_forward_navigation() {
        let mut model = InstallerModel::default();
        model.advance().unwrap();
        assert_eq!(
            model.advance(),
            Err(InstallerError::MissingSelection(InstallerStep::Language))
        );
    }

    #[test]
    fn dry_run_reaches_completion_without_destructive_confirmation() {
        let mut model = ready_model(InstallMode::DryRun);
        assert_eq!(model.confirmation_phrase(), None);
        model.begin_install().unwrap();
        assert_eq!(model.step(), InstallerStep::Installation);
        model.complete_install().unwrap();
        assert_eq!(model.step(), InstallerStep::Completed);
    }

    #[test]
    fn real_install_requires_exact_target_bound_confirmation() {
        let mut model = ready_model(InstallMode::Real);
        assert_eq!(
            model.confirmation_phrase().as_deref(),
            Some("ERASE /dev/vda")
        );
        assert_eq!(
            model.begin_install(),
            Err(InstallerError::DestructiveConfirmationRequired)
        );
        assert_eq!(
            model.confirm_destructive("ERASE /dev/sda"),
            Err(InstallerError::ConfirmationPhraseMismatch)
        );
        model.confirm_destructive("ERASE /dev/vda").unwrap();
        model.begin_install().unwrap();
    }

    #[test]
    fn changing_target_invalidates_confirmation() {
        let mut model = ready_model(InstallMode::Real);
        model.confirm_destructive("ERASE /dev/vda").unwrap();
        assert!(model.destructive_confirmed());
        model.set_target(InstallTarget::erase_disk(disk("/dev/vdb", "virtio-other")));
        assert!(!model.destructive_confirmed());
        assert_eq!(
            model.confirmation_phrase().as_deref(),
            Some("ERASE /dev/vdb")
        );
    }

    #[test]
    fn changing_mode_invalidates_confirmation() {
        let mut model = ready_model(InstallMode::Real);
        model.confirm_destructive("ERASE /dev/vda").unwrap();
        model.set_mode(InstallMode::DryRun);
        assert!(!model.destructive_confirmed());
    }

    #[test]
    fn installation_and_completion_cannot_navigate_back() {
        let mut model = ready_model(InstallMode::DryRun);
        model.begin_install().unwrap();
        assert_eq!(
            model.retreat(),
            Err(InstallerError::CannotMoveBack(InstallerStep::Installation))
        );
        model.complete_install().unwrap();
        assert_eq!(
            model.retreat(),
            Err(InstallerError::CannotMoveBack(InstallerStep::Completed))
        );
    }

    #[test]
    fn user_profile_validates_account_contract_without_storing_a_password() {
        assert_eq!(
            UserProfile::new("Admin", "Administrator", true),
            Err(InstallerError::InvalidUserProfile)
        );
        assert_eq!(
            UserProfile::new("aqua", "Aqua User", false),
            Err(InstallerError::InvalidUserProfile)
        );
        let user = UserProfile::new("aqua_1", "Aqua User", true).unwrap();
        assert_eq!(user.username(), "aqua_1");
        assert!(user.password_configured());
    }

    #[test]
    fn disk_identity_rejects_ambiguous_or_empty_targets() {
        assert_eq!(
            DiskIdentity::new("vda", "virtio-aqua", "QEMU", 1024),
            Err(InstallerError::InvalidDiskIdentity)
        );
        assert_eq!(
            DiskIdentity::new("/dev/vda", "", "QEMU", 1024),
            Err(InstallerError::InvalidDiskIdentity)
        );
        assert_eq!(
            DiskIdentity::new("/dev/vda", "virtio-aqua", "QEMU", 0),
            Err(InstallerError::InvalidDiskIdentity)
        );
    }

    #[test]
    fn summary_requires_begin_install_entrypoint() {
        let mut model = ready_model(InstallMode::DryRun);
        assert_eq!(model.advance(), Err(InstallerError::BeginInstallRequired));
        assert_eq!(model.step(), InstallerStep::Summary);
    }

    fn test_artifacts() -> InstallArtifacts {
        InstallArtifacts::new(
            "/run/aqua-installer/rootfs.tar",
            "/run/aqua-installer/bzImage",
            "/run/aqua-installer/efi-part/EFI/BOOT/bootx64.efi",
        )
        .unwrap()
    }

    #[test]
    fn dry_run_plan_is_deterministic_and_never_executable() {
        let model = ready_model(InstallMode::DryRun);
        let first = build_dry_run_plan(&model, &test_artifacts()).unwrap();
        let second = build_dry_run_plan(&model, &test_artifacts()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.fingerprint(), second.fingerprint());
        assert!(!first.execution_allowed());
        assert_eq!(first.operations().len(), 13);
        assert!(first.blockers().is_empty());
        let rendered = first.render();
        assert!(rendered.contains("plan_mode=dry-run"));
        assert!(rendered.contains("execution_allowed=false"));
        assert!(rendered.contains("write-gpt device=/dev/vda"));
        assert!(rendered.contains("partition=/dev/vda1"));
        assert!(rendered.contains("partition=/dev/vda2"));
        assert!(rendered.contains("label=AQUA_EFI"));
        assert!(rendered.contains("label=AQUA_ROOT"));
        assert!(rendered.contains("strategy=grub2-x86_64-efi"));
        assert!(rendered.contains("destination=/mnt/aqua-target/boot/efi/EFI/BOOT/BOOTX64.EFI"));
        assert!(rendered.contains("config_destination=/mnt/aqua-target/boot/efi/EFI/BOOT/grub.cfg"));
        assert!(rendered.contains("kernel_cmdline=root%3DPARTLABEL%3DAQUA_ROOT"));
        assert!(rendered.contains("blocker_count=0"));
        assert!(rendered.contains("executed=false"));
    }

    #[test]
    fn dry_run_plan_handles_digit_terminated_device_names() {
        assert_eq!(partition_path("/dev/nvme0n1", 1), "/dev/nvme0n1p1");
        assert_eq!(partition_path("/dev/mmcblk0", 2), "/dev/mmcblk0p2");
        assert_eq!(partition_path("/dev/vda", 2), "/dev/vda2");
    }

    #[test]
    fn dry_run_plan_contains_configuration_but_no_password_contents() {
        let model = ready_model(InstallMode::DryRun);
        let rendered = build_dry_run_plan(&model, &test_artifacts())
            .unwrap()
            .render();
        assert!(rendered.contains("locale=tr_TR.UTF-8"));
        assert!(rendered.contains("timezone=Europe/Istanbul"));
        assert!(rendered.contains("username=aqua"));
        assert!(rendered.contains("password_configured=true"));
        assert!(!rendered.to_ascii_lowercase().contains("password="));
        assert!(!rendered.contains("secret"));
    }

    #[test]
    fn dry_run_plan_requires_summary_and_safe_artifact_paths() {
        let model = InstallerModel::default();
        assert_eq!(
            build_dry_run_plan(&model, &test_artifacts()),
            Err(InstallPlanError::NotAtSummary)
        );
        assert_eq!(
            InstallArtifacts::new(
                "relative/rootfs.tar",
                "/run/aqua-installer/bzImage",
                "/run/aqua-installer/bootx64.efi"
            ),
            Err(InstallPlanError::InvalidArtifactPath("rootfs-archive"))
        );
        assert_eq!(
            InstallArtifacts::new(
                "/run/aqua-installer/../rootfs.tar",
                "/run/aqua-installer/bzImage",
                "/run/aqua-installer/bootx64.efi"
            ),
            Err(InstallPlanError::InvalidArtifactPath("rootfs-archive"))
        );
        assert_eq!(
            InstallArtifacts::new(
                "/run/aqua-installer/rootfs.tar",
                "/run/aqua-installer/bzImage",
                "relative/bootx64.efi"
            ),
            Err(InstallPlanError::InvalidArtifactPath("bootloader-image"))
        );
    }

    #[test]
    fn dry_run_plan_rejects_target_below_minimum_capacity() {
        let mut model = ready_model(InstallMode::DryRun);
        model.set_target(InstallTarget::erase_disk(
            DiskIdentity::new(
                "/dev/vdz",
                "small-target",
                "Small test disk",
                INSTALL_MINIMUM_CAPACITY_BYTES - 1,
            )
            .unwrap(),
        ));
        assert_eq!(
            build_dry_run_plan(&model, &test_artifacts()),
            Err(InstallPlanError::InsufficientCapacity {
                available_bytes: INSTALL_MINIMUM_CAPACITY_BYTES - 1,
                required_bytes: INSTALL_MINIMUM_CAPACITY_BYTES,
            })
        );
    }

    #[test]
    fn dry_run_fingerprint_changes_with_plan_inputs() {
        let first_model = ready_model(InstallMode::DryRun);
        let mut second_model = first_model.clone();
        second_model.set_timezone("UTC").unwrap();
        let first = build_dry_run_plan(&first_model, &test_artifacts()).unwrap();
        let second = build_dry_run_plan(&second_model, &test_artifacts()).unwrap();
        assert_ne!(first.fingerprint(), second.fingerprint());
        assert_ne!(first.render(), second.render());
    }

    #[test]
    fn real_mode_still_produces_non_executing_preview_plan() {
        let model = ready_model(InstallMode::Real);
        let plan = build_dry_run_plan(&model, &test_artifacts()).unwrap();
        assert_eq!(plan.source_mode(), InstallMode::Real);
        assert!(!plan.execution_allowed());
        assert!(plan.render().contains("source_mode=real"));
        assert!(plan.render().contains("execution_allowed=false"));
    }

    fn test_tool_paths(root: &Path) -> InstallToolPaths {
        InstallToolPaths {
            sfdisk: root.join("sbin/sfdisk"),
            mkfs_fat: root.join("sbin/mkfs.fat"),
            mkfs_ext4: root.join("sbin/mkfs.ext4"),
            tar: root.join("bin/tar"),
            mount: root.join("bin/mount"),
            umount: root.join("bin/umount"),
        }
    }

    #[test]
    fn install_prerequisites_require_all_executable_absolute_tools() {
        let root = TestStorageRoot::new();
        let paths = test_tool_paths(&root.path);
        for (_, path) in paths.entries() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "#!/bin/sh\nexit 0\n").unwrap();
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }

        let prerequisites = validate_install_prerequisites(&paths).unwrap();
        assert_eq!(prerequisites.tools().len(), 6);
        assert_eq!(prerequisites.tools()[0].tool(), InstallTool::Sfdisk);
        assert_eq!(prerequisites.tools()[5].tool(), InstallTool::Umount);

        let mut permissions = fs::metadata(&paths.mkfs_fat).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&paths.mkfs_fat, permissions).unwrap();
        assert_eq!(
            validate_install_prerequisites(&paths),
            Err(InstallPrerequisiteError::NotExecutable(
                InstallTool::MkfsFat
            ))
        );
    }

    #[test]
    fn install_prerequisites_fail_closed_for_missing_or_relative_tools() {
        let root = TestStorageRoot::new();
        let paths = test_tool_paths(&root.path);
        assert_eq!(
            validate_install_prerequisites(&paths),
            Err(InstallPrerequisiteError::Missing(InstallTool::Sfdisk))
        );

        let mut relative = paths;
        relative.sfdisk = PathBuf::from("sbin/sfdisk");
        assert_eq!(
            validate_install_prerequisites(&relative),
            Err(InstallPrerequisiteError::InvalidPath(InstallTool::Sfdisk))
        );
    }

    fn validated_test_prerequisites(root: &TestStorageRoot) -> InstallPrerequisites {
        let paths = test_tool_paths(&root.path);
        for (_, path) in paths.entries() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "#!/bin/sh\nexit 0\n").unwrap();
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
        validate_install_prerequisites(&paths).unwrap()
    }

    #[test]
    fn command_compiler_produces_bounded_argv_without_shell_dispatch() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::Real), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();

        assert_eq!(commands.source_fingerprint(), source.fingerprint());
        assert_eq!(commands.commands().len(), 8);
        assert!(!commands.execution_allowed());
        assert_eq!(
            commands.deferred_operations(),
            [
                "verify-target",
                "install-kernel",
                "install-bootloader",
                "write-system-configuration",
            ]
        );
        let partition = &commands.commands()[0];
        assert_eq!(partition.tool(), InstallTool::Sfdisk);
        assert_eq!(partition.operation(), "write-partition-table");
        assert_eq!(partition.arguments().last().unwrap(), "/dev/vda");
        assert_eq!(
            partition.stdin(),
            Some("label: gpt\nunit: sectors\n\nstart=2048, size=1048576, type=U, name=\"AQUA_EFI\"\nstart=1050624, type=L, name=\"AQUA_ROOT\"\n")
        );
        assert_eq!(commands.commands()[1].tool(), InstallTool::MkfsFat);
        assert_eq!(commands.commands()[2].tool(), InstallTool::MkfsExt4);
        assert_eq!(commands.commands()[3].tool(), InstallTool::Mount);
        assert_eq!(commands.commands()[4].tool(), InstallTool::Tar);
        assert_eq!(commands.commands()[5].tool(), InstallTool::Mount);
        assert_eq!(commands.commands()[6].tool(), InstallTool::Umount);
        assert_eq!(commands.commands()[7].tool(), InstallTool::Umount);
        assert!(commands
            .commands()
            .iter()
            .all(|command| command.arguments().len() <= INSTALL_ARGUMENT_LIMIT));
    }

    #[test]
    fn non_executing_runner_records_rehearsal_without_process_execution() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let mut runner = NonExecutingInstallCommandRunner::default();

        let rehearsal = runner.rehearse(&commands);
        assert_eq!(rehearsal.source_fingerprint(), source.fingerprint());
        assert_eq!(rehearsal.command_count(), 8);
        assert!(!rehearsal.executed());
        assert_eq!(runner.rehearsed(), commands.commands());
    }

    #[test]
    fn command_compiler_rejects_missing_tools_and_changed_geometry() {
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        assert_eq!(
            compile_install_commands(&source, &InstallPrerequisites { tools: Vec::new() },),
            Err(InstallCommandCompileError::MissingTool(InstallTool::Sfdisk))
        );

        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let mut changed = source;
        changed.operations[2] = InstallPlanOperation::CreateEfiSystemPartition {
            partition: "/dev/vda1".to_string(),
            start_mib: 2,
            size_mib: INSTALL_ESP_SIZE_MIB,
        };
        assert_eq!(
            compile_install_commands(&changed, &prerequisites),
            Err(InstallCommandCompileError::InvalidPlan(
                "partition geometry or identity changed"
            ))
        );
    }

    #[test]
    fn command_spec_rejects_control_characters_and_unbounded_stdin() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        assert_eq!(
            install_command(
                &prerequisites,
                "test",
                InstallTool::Mount,
                vec!["/dev/vda2\nunsafe".to_string()],
                None,
            ),
            Err(InstallCommandCompileError::UnsafeArgument("test"))
        );
        assert_eq!(
            install_command(
                &prerequisites,
                "test",
                InstallTool::Sfdisk,
                vec!["/dev/vda".to_string()],
                Some("x".repeat(INSTALL_STDIN_BYTES_LIMIT + 1)),
            ),
            Err(InstallCommandCompileError::UnsafeStdin)
        );
    }

    #[test]
    fn internal_plan_is_bounded_target_confined_and_atomic() {
        let source =
            build_dry_run_plan(&ready_model(InstallMode::Real), &test_artifacts()).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();

        assert_eq!(internal.source_fingerprint(), source.fingerprint());
        assert_eq!(internal.actions().len(), 11);
        assert!(!internal.execution_allowed());
        assert_eq!(
            internal
                .actions()
                .iter()
                .filter(|action| action.anchor() == InternalInstallAnchor::BeforeMountRoot)
                .count(),
            1
        );
        assert_eq!(
            internal
                .actions()
                .iter()
                .filter(|action| action.anchor() == InternalInstallAnchor::BeforeMountEfi)
                .count(),
            1
        );
        assert_eq!(
            internal
                .actions()
                .iter()
                .filter(|action| action.anchor() == InternalInstallAnchor::AfterMountEfi)
                .count(),
            9
        );

        for action in internal.actions() {
            match action.kind() {
                InternalInstallActionKind::CreateDirectory { path, mode } => {
                    assert!(path.starts_with(INSTALL_TARGET_ROOT));
                    assert_eq!(*mode, 0o755);
                }
                InternalInstallActionKind::CopyFileAtomic {
                    source,
                    destination,
                    temporary,
                    mode,
                } => {
                    assert!(source.is_absolute());
                    assert!(destination.starts_with(INSTALL_TARGET_ROOT));
                    assert_eq!(destination.parent(), temporary.parent());
                    assert_ne!(destination, temporary);
                    assert_eq!(*mode, 0o644);
                }
                InternalInstallActionKind::WriteFileAtomic {
                    destination,
                    temporary,
                    content,
                    mode,
                } => {
                    assert!(destination.starts_with(INSTALL_TARGET_ROOT));
                    assert_eq!(destination.parent(), temporary.parent());
                    assert_ne!(destination, temporary);
                    assert!(content.len() <= INTERNAL_INSTALL_CONTENT_LIMIT);
                    assert_eq!(*mode, 0o644);
                }
            }
        }
    }

    #[test]
    fn internal_plan_generates_grub_and_password_free_system_metadata() {
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let writes = internal.actions().iter().filter_map(|action| {
            let InternalInstallActionKind::WriteFileAtomic {
                destination,
                content,
                ..
            } = action.kind()
            else {
                return None;
            };
            Some((destination.as_path(), content.as_str()))
        });
        let writes = writes.collect::<Vec<_>>();

        let grub = writes
            .iter()
            .find(|(path, _)| path.ends_with("EFI/BOOT/grub.cfg"))
            .unwrap()
            .1;
        assert!(grub.contains("search --no-floppy --label AQUA_ROOT --set=root"));
        assert!(grub.contains("linux /boot/vmlinuz-aqua root=PARTLABEL=AQUA_ROOT"));
        assert!(writes.iter().any(
            |(path, content)| path.ends_with("locale.conf") && *content == "LANG=tr_TR.UTF-8\n"
        ));
        assert!(writes
            .iter()
            .any(|(path, content)| path.ends_with("first-user.conf")
                && content.contains("display_name=Aqua%20User")
                && content.contains("password_configured=true")
                && !content.contains("password=")));
    }

    #[test]
    fn internal_rehearsal_records_actions_without_filesystem_writes() {
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let mut runner = NonExecutingInternalInstallRunner::default();

        let rehearsal = runner.rehearse(&internal);
        assert_eq!(rehearsal.source_fingerprint(), source.fingerprint());
        assert_eq!(rehearsal.action_count(), 11);
        assert!(!rehearsal.executed());
        assert_eq!(runner.rehearsed(), internal.actions());
    }

    #[test]
    fn internal_plan_rejects_destinations_outside_target_root() {
        let mut source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        source.operations[9] = InstallPlanOperation::InstallKernel {
            source: PathBuf::from("/run/aqua-installer/bzImage"),
            destination: "/boot/vmlinuz-aqua",
        };
        assert_eq!(
            compile_internal_install_actions(&source),
            Err(InternalInstallCompileError::UnsafePath("kernel-copy"))
        );
    }

    #[test]
    fn transaction_graph_interleaves_revalidation_commands_and_internal_actions() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::Real), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();

        assert_eq!(graph.source_fingerprint(), source.fingerprint());
        assert_eq!(graph.steps().len(), 20);
        assert_eq!(graph.cleanup().len(), 2);
        assert!(!graph.execution_allowed());
        assert!(matches!(
            &graph.steps()[0],
            InstallTransactionStep::RevalidateTarget { expected }
                if expected.device() == "/dev/vda"
        ));
        assert!(matches!(
            &graph.steps()[1],
            InstallTransactionStep::Internal(action)
                if action.anchor() == InternalInstallAnchor::BeforeMountRoot
        ));
        assert!(matches!(
            &graph.steps()[5],
            InstallTransactionStep::Command(command)
                if command.operation() == "mount-root"
        ));
        assert!(matches!(
            &graph.steps()[8],
            InstallTransactionStep::Command(command)
                if command.operation() == "mount-efi-system-partition"
        ));
        assert_eq!(
            graph.cleanup()[0].requirement(),
            InstallCleanupRequirement::EfiMounted
        );
        assert_eq!(
            graph.cleanup()[1].requirement(),
            InstallCleanupRequirement::RootMounted
        );
    }

    #[test]
    fn transaction_progress_is_monotonic_and_bound_to_real_steps() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::Real), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();

        let events = (0..graph.steps().len())
            .map(|completed| InstallProgressEvent::running(&graph, completed).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 20);
        assert_eq!(events[0].state(), InstallProgressState::Running);
        assert_eq!(events[0].phase(), InstallProgressPhase::PreparingTarget);
        assert_eq!(events[0].operation(), "revalidate-target");
        assert_eq!(events[0].percent(), 0);
        assert_eq!(events[2].phase(), InstallProgressPhase::Partitioning);
        assert_eq!(events[3].phase(), InstallProgressPhase::Formatting);
        assert_eq!(events[6].phase(), InstallProgressPhase::InstallingSystem);
        assert_eq!(
            events[8].phase(),
            InstallProgressPhase::InstallingBootloader
        );
        assert_eq!(events[13].phase(), InstallProgressPhase::ConfiguringSystem);
        assert_eq!(events[18].phase(), InstallProgressPhase::Finalizing);
        assert_eq!(events[19].percent(), 95);
        assert!(events
            .windows(2)
            .all(|pair| pair[0].percent() < pair[1].percent()));

        let completed = InstallProgressEvent::completed(&graph).unwrap();
        assert_eq!(completed.state(), InstallProgressState::Completed);
        assert_eq!(completed.phase(), InstallProgressPhase::Completed);
        assert_eq!(completed.operation(), "complete");
        assert_eq!(completed.completed_steps(), 20);
        assert_eq!(completed.total_steps(), 20);
        assert_eq!(completed.percent(), 100);
    }

    #[test]
    fn failed_transaction_progress_preserves_last_completed_step() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::Real), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();

        let failed = InstallProgressEvent::failed(&graph, 9).unwrap();
        assert_eq!(failed.state(), InstallProgressState::Failed);
        assert_eq!(failed.phase(), InstallProgressPhase::InstallingBootloader);
        assert_eq!(failed.operation(), "prepare-bootloader-directory");
        assert_eq!(failed.completed_steps(), 9);
        assert_eq!(failed.percent(), 45);
        assert_eq!(
            InstallProgressEvent::running(&graph, 20),
            Err(InstallProgressError::InvalidCompletedStepCount {
                completed: 20,
                total: 20,
            })
        );
    }

    #[test]
    fn transaction_rehearsal_schedules_only_mounted_cleanup_in_reverse_order() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();
        let runner = NonExecutingInstallTransactionRunner;

        let before_root_mount = runner.rehearse(&graph, Some(5)).unwrap();
        assert!(before_root_mount.cleanup().is_empty());

        let after_root_mount = runner.rehearse(&graph, Some(6)).unwrap();
        assert_eq!(after_root_mount.cleanup().len(), 1);
        assert_eq!(
            after_root_mount.cleanup()[0].requirement(),
            InstallCleanupRequirement::RootMounted
        );

        let after_efi_mount = runner.rehearse(&graph, Some(9)).unwrap();
        assert_eq!(after_efi_mount.cleanup().len(), 2);
        assert_eq!(
            after_efi_mount.cleanup()[0].requirement(),
            InstallCleanupRequirement::EfiMounted
        );
        assert_eq!(
            after_efi_mount.cleanup()[1].requirement(),
            InstallCleanupRequirement::RootMounted
        );
        assert!(!after_efi_mount.executed());

        let after_normal_efi_unmount = runner.rehearse(&graph, Some(19)).unwrap();
        assert_eq!(after_normal_efi_unmount.cleanup().len(), 1);
        assert_eq!(
            after_normal_efi_unmount.cleanup()[0].requirement(),
            InstallCleanupRequirement::RootMounted
        );
    }

    #[test]
    fn transaction_rehearsal_completes_without_cleanup_or_execution() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();
        let runner = NonExecutingInstallTransactionRunner;

        let rehearsal = runner.rehearse(&graph, None).unwrap();
        assert_eq!(
            rehearsal.outcome(),
            InstallTransactionRehearsalOutcome::Completed
        );
        assert_eq!(rehearsal.rehearsed_steps().len(), 20);
        assert!(rehearsal.cleanup().is_empty());
        assert!(!rehearsal.executed());
        assert_eq!(
            runner.rehearse(&graph, Some(20)),
            Err(InstallTransactionRehearsalError::InvalidFailureIndex(20))
        );
    }

    #[test]
    fn transaction_graph_rejects_mismatched_rehearsal_sources() {
        let root = TestStorageRoot::new();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let mut internal = compile_internal_install_actions(&source).unwrap();
        internal.source_fingerprint ^= 1;

        assert_eq!(
            build_install_transaction_graph(&source, &commands, &internal),
            Err(InstallTransactionCompileError::FingerprintMismatch)
        );
    }

    fn fixture_artifacts(root: &TestStorageRoot) -> (InstallArtifacts, PathBuf, PathBuf) {
        let sources = root.path.join("fixture-sources");
        fs::create_dir_all(&sources).unwrap();
        let rootfs = sources.join("rootfs.tar");
        let kernel = sources.join("bzImage");
        let bootloader = sources.join("bootx64.efi");
        fs::write(&rootfs, b"unused-rootfs-fixture").unwrap();
        fs::write(&kernel, b"aqua-kernel-fixture").unwrap();
        fs::write(&bootloader, b"aqua-efi-fixture").unwrap();
        (
            InstallArtifacts::new(&rootfs, &kernel, &bootloader).unwrap(),
            kernel,
            bootloader,
        )
    }

    #[test]
    fn fixture_executor_performs_only_root_remapped_internal_actions() {
        let root = TestStorageRoot::new();
        let fixture_path = root.path.join("fixture-target");
        fs::create_dir(&fixture_path).unwrap();
        let fixture = FixtureInstallRoot::new(&fixture_path).unwrap();
        let (artifacts, kernel_source, bootloader_source) = fixture_artifacts(&root);
        let source = build_dry_run_plan(&ready_model(InstallMode::DryRun), &artifacts).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();

        let report = FixtureInternalInstallExecutor
            .execute(&fixture, &internal)
            .unwrap();
        assert_eq!(report.source_fingerprint(), source.fingerprint());
        assert_eq!(report.action_count(), 11);
        assert!(report.fixture_filesystem_executed());
        assert!(!report.disk_commands_executed());
        assert_eq!(
            fs::read(fixture.path().join("boot/vmlinuz-aqua")).unwrap(),
            fs::read(kernel_source).unwrap()
        );
        assert_eq!(
            fs::read(fixture.path().join("boot/efi/EFI/BOOT/BOOTX64.EFI")).unwrap(),
            fs::read(bootloader_source).unwrap()
        );
        assert!(
            fs::read_to_string(fixture.path().join("boot/efi/EFI/BOOT/grub.cfg"))
                .unwrap()
                .contains("root=PARTLABEL=AQUA_ROOT")
        );
        assert_eq!(
            fs::read_to_string(fixture.path().join("etc/locale.conf")).unwrap(),
            "LANG=tr_TR.UTF-8\n"
        );
        assert_eq!(
            fs::read_to_string(fixture.path().join("etc/vconsole.conf")).unwrap(),
            "KEYMAP=tr\n"
        );
        assert_eq!(
            fs::read_to_string(fixture.path().join("etc/timezone")).unwrap(),
            "Europe/Istanbul\n"
        );
        let user = fs::read_to_string(fixture.path().join("etc/aqua/first-user.conf")).unwrap();
        assert!(user.contains("display_name=Aqua%20User"));
        assert!(!user.contains("password="));
        assert!(fs::read_dir(fixture.path())
            .unwrap()
            .flat_map(|entry| entry.ok())
            .all(|entry| !entry
                .file_name()
                .to_string_lossy()
                .contains("aqua-install.tmp")));
    }

    #[test]
    fn fixture_root_requires_an_empty_real_directory_under_system_temp() {
        let root = TestStorageRoot::new();
        assert_eq!(
            FixtureInstallRoot::new("/"),
            Err(FixtureInstallError::InvalidRoot)
        );
        let nonempty = root.path.join("nonempty-fixture");
        fs::create_dir(&nonempty).unwrap();
        fs::write(nonempty.join("existing"), b"data").unwrap();
        assert_eq!(
            FixtureInstallRoot::new(nonempty),
            Err(FixtureInstallError::RootNotEmpty)
        );

        let real = root.path.join("real-fixture");
        let link = root.path.join("linked-fixture");
        fs::create_dir(&real).unwrap();
        symlink(&real, &link).unwrap();
        assert_eq!(
            FixtureInstallRoot::new(link),
            Err(FixtureInstallError::InvalidRoot)
        );
    }

    #[test]
    fn fixture_executor_rejects_symlink_escape_components() {
        let root = TestStorageRoot::new();
        let fixture_path = root.path.join("fixture-target");
        let outside = root.path.join("outside-target");
        fs::create_dir(&fixture_path).unwrap();
        fs::create_dir(&outside).unwrap();
        let fixture = FixtureInstallRoot::new(&fixture_path).unwrap();
        symlink(&outside, fixture.path().join("boot")).unwrap();
        let (artifacts, _, _) = fixture_artifacts(&root);
        let source = build_dry_run_plan(&ready_model(InstallMode::DryRun), &artifacts).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();

        assert_eq!(
            FixtureInternalInstallExecutor.execute(&fixture, &internal),
            Err(FixtureInstallError::SymlinkComponent(
                fixture.path().join("boot")
            ))
        );
        assert!(fs::read_dir(outside).unwrap().next().is_none());
    }

    #[test]
    fn fixture_executor_rejects_symlink_artifact_sources() {
        let root = TestStorageRoot::new();
        let fixture_path = root.path.join("fixture-target");
        fs::create_dir(&fixture_path).unwrap();
        let fixture = FixtureInstallRoot::new(&fixture_path).unwrap();
        let (artifacts, kernel, _) = fixture_artifacts(&root);
        let real_kernel = kernel.with_extension("real");
        fs::rename(&kernel, &real_kernel).unwrap();
        symlink(&real_kernel, &kernel).unwrap();
        let source = build_dry_run_plan(&ready_model(InstallMode::DryRun), &artifacts).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();

        assert_eq!(
            FixtureInternalInstallExecutor.execute(&fixture, &internal),
            Err(FixtureInstallError::InvalidSource(kernel))
        );
    }

    fn write_fixture_shim(path: &Path, exit_code: i32) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                "#!/bin/sh\nname=${{0##*/}}\nlog=\"$AQUA_FIXTURE_LOG_DIR/$name.log\"\nprintf 'call\\n' >> \"$log\"\nfor arg do printf 'arg=%s\\n' \"$arg\" >> \"$log\"; done\nwhile IFS= read -r line || [ -n \"$line\" ]; do printf 'stdin=%s\\n' \"$line\" >> \"$log\"; done\nexit {exit_code}\n"
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn fixture_shim_graph(
        root: &TestStorageRoot,
        failing_tool: Option<InstallTool>,
    ) -> (FixtureToolShimRoot, InstallTransactionGraph) {
        let shim_path = root.path.join("fixture-tools");
        fs::create_dir(&shim_path).unwrap();
        let paths = test_tool_paths(&shim_path);
        for (tool, path) in paths.entries() {
            write_fixture_shim(path, if failing_tool == Some(tool) { 7 } else { 0 });
        }
        let prerequisites = validate_install_prerequisites(&paths).unwrap();
        let shim_root = FixtureToolShimRoot::new(&shim_path).unwrap();
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();
        (shim_root, graph)
    }

    #[test]
    fn fixture_tool_runner_delivers_exact_argv_and_bounded_stdin() {
        let root = TestStorageRoot::new();
        let (shim_root, graph) = fixture_shim_graph(&root, None);

        let report = FixtureToolShimRunner
            .execute_transaction_commands(&shim_root, &graph)
            .unwrap();
        assert_eq!(report.outcome(), &FixtureToolShimOutcome::Completed);
        assert_eq!(report.completed_commands().len(), 8);
        assert!(report.cleanup().is_empty());
        assert!(report.shim_processes_executed());
        assert!(!report.real_disk_tools_executed());
        let sfdisk_log = fs::read_to_string(shim_root.log_directory().join("sfdisk.log")).unwrap();
        assert!(sfdisk_log.contains("arg=--wipe\narg=always\n"));
        assert!(sfdisk_log.contains("arg=/dev/vda\n"));
        assert!(sfdisk_log.contains("stdin=label: gpt\n"));
        assert!(sfdisk_log.contains("stdin=unit: sectors\n"));
        assert!(sfdisk_log.contains("stdin=start=2048, size=1048576, type=U, name=\"AQUA_EFI\"\n"));
        let mount_log = fs::read_to_string(shim_root.log_directory().join("mount.log")).unwrap();
        assert_eq!(mount_log.matches("call\n").count(), 2);
        assert!(mount_log.contains("arg=/dev/vda2\narg=/mnt/aqua-target\n"));
        assert!(mount_log.contains("arg=/dev/vda1\narg=/mnt/aqua-target/boot/efi\n"));
    }

    #[test]
    fn fixture_tool_failure_propagates_transaction_cleanup_state() {
        let root = TestStorageRoot::new();
        let (shim_root, graph) = fixture_shim_graph(&root, Some(InstallTool::Tar));

        let report = FixtureToolShimRunner
            .execute_transaction_commands(&shim_root, &graph)
            .unwrap();
        assert_eq!(
            report.outcome(),
            &FixtureToolShimOutcome::Failed {
                transaction_step: 6,
                operation: "extract-root-filesystem",
                exit_code: Some(7),
                timed_out: false,
            }
        );
        assert_eq!(report.completed_commands().len(), 4);
        assert_eq!(report.cleanup().len(), 1);
        assert_eq!(
            report.cleanup()[0].requirement(),
            InstallCleanupRequirement::RootMounted
        );
        assert!(!report.real_disk_tools_executed());
    }

    #[test]
    fn fixture_tool_timeout_kills_process_and_preserves_cleanup_plan() {
        let root = TestStorageRoot::new();
        let (shim_root, graph) = fixture_shim_graph(&root, None);
        let tar = shim_root.path().join("bin/tar");
        fs::write(&tar, "#!/bin/sh\nwhile :; do :; done\n").unwrap();
        let mut permissions = fs::metadata(&tar).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&tar, permissions).unwrap();

        let report = FixtureToolShimRunner
            .execute_transaction_commands(&shim_root, &graph)
            .unwrap();
        assert_eq!(
            report.outcome(),
            &FixtureToolShimOutcome::Failed {
                transaction_step: 6,
                operation: "extract-root-filesystem",
                exit_code: None,
                timed_out: true,
            }
        );
        assert_eq!(report.cleanup().len(), 1);
        assert_eq!(
            report.cleanup()[0].requirement(),
            InstallCleanupRequirement::RootMounted
        );
    }

    #[test]
    fn fixture_tool_runner_rejects_programs_outside_capability_root() {
        let root = TestStorageRoot::new();
        let shim_path = root.path.join("shim-capability");
        fs::create_dir(&shim_path).unwrap();
        let shim_root = FixtureToolShimRoot::new(&shim_path).unwrap();
        let prerequisites = validated_test_prerequisites(&root);
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();

        assert_eq!(
            FixtureToolShimRunner.execute_transaction_commands(&shim_root, &graph),
            Err(FixtureToolShimError::ProgramOutsideRoot(
                root.path.join("sbin/sfdisk")
            ))
        );
    }

    #[test]
    fn fixture_tool_runner_rejects_symlink_programs() {
        let root = TestStorageRoot::new();
        let shim_path = root.path.join("fixture-tools");
        fs::create_dir(&shim_path).unwrap();
        let paths = test_tool_paths(&shim_path);
        for (_, path) in paths.entries() {
            write_fixture_shim(path, 0);
        }
        let prerequisites = validate_install_prerequisites(&paths).unwrap();
        let real_sfdisk = paths.sfdisk.with_extension("real");
        fs::rename(&paths.sfdisk, &real_sfdisk).unwrap();
        symlink(&real_sfdisk, &paths.sfdisk).unwrap();
        let shim_root = FixtureToolShimRoot::new(&shim_path).unwrap();
        let source =
            build_dry_run_plan(&ready_model(InstallMode::DryRun), &test_artifacts()).unwrap();
        let commands = compile_install_commands(&source, &prerequisites).unwrap();
        let internal = compile_internal_install_actions(&source).unwrap();
        let graph = build_install_transaction_graph(&source, &commands, &internal).unwrap();

        assert_eq!(
            FixtureToolShimRunner.execute_transaction_commands(&shim_root, &graph),
            Err(FixtureToolShimError::SymlinkComponent(paths.sfdisk))
        );
    }

    #[test]
    fn target_revalidation_requires_identical_eligible_disk() {
        let root = TestStorageRoot::new();
        root.add_disk("vdb", "252:16", 67_108_864, false);
        let expected = disk("/dev/vdb", "aqua-vdb");

        let verified = revalidate_install_target(&root.paths(), &expected).unwrap();
        assert_eq!(verified.device(), "/dev/vdb");
        assert_eq!(verified.stable_id(), "aqua-vdb");

        let changed = disk("/dev/vdb", "different-serial");
        assert_eq!(
            revalidate_install_target(&root.paths(), &changed),
            Err(TargetRevalidationError::IdentityChanged {
                device: "/dev/vdb".to_string(),
                field: "stable-id",
            })
        );
    }

    #[test]
    fn target_revalidation_rejects_disappeared_or_newly_blocked_disk() {
        let missing_root = TestStorageRoot::new();
        let expected = disk("/dev/vdb", "aqua-vdb");
        assert_eq!(
            revalidate_install_target(&missing_root.paths(), &expected),
            Err(TargetRevalidationError::Missing("/dev/vdb".to_string()))
        );

        let blocked_root = TestStorageRoot::new();
        blocked_root.add_disk("vdb", "252:16", 67_108_864, true);
        assert_eq!(
            revalidate_install_target(&blocked_root.paths(), &expected),
            Err(TargetRevalidationError::Blocked {
                device: "/dev/vdb".to_string(),
                reasons: vec![StorageBlockReason::ReadOnly],
            })
        );
    }

    #[test]
    fn storage_probe_blocks_running_root_disk_and_keeps_safe_target() {
        let root = TestStorageRoot::new();
        root.add_disk("vda", "252:0", 67_108_864, false);
        root.add_partition("vda", "vda1", "252:1");
        root.add_disk("vdb", "252:16", 67_108_864, false);
        fs::write(
            root.path.join("proc/self/mountinfo"),
            "29 23 252:1 / / rw - ext4 /dev/vda1 rw\n",
        )
        .unwrap();

        let inventory = probe_storage(&root.paths()).unwrap();
        assert_eq!(inventory.candidates().len(), 2);
        assert_eq!(inventory.root_device_names(), &["vda1"]);
        let system = inventory
            .candidates()
            .iter()
            .find(|candidate| candidate.device() == "/dev/vda")
            .unwrap();
        assert_eq!(
            system.blocked_reasons(),
            &[StorageBlockReason::RunningSystemDisk]
        );
        assert!(matches!(
            system.clone().into_erase_target(),
            Err(StorageProbeError::BlockedTarget { .. })
        ));

        let safe = inventory
            .eligible_candidates()
            .find(|candidate| candidate.device() == "/dev/vdb")
            .unwrap();
        let target = safe.clone().into_erase_target().unwrap();
        assert_eq!(target.disk.device(), "/dev/vdb");
        assert_eq!(target.disk.stable_id(), "aqua-vdb");
    }

    #[test]
    fn storage_probe_uses_kernel_root_argument_when_rootfs_is_memory_backed() {
        let root = TestStorageRoot::new();
        root.add_disk("nvme0n1", "259:0", 134_217_728, false);
        root.add_partition("nvme0n1", "nvme0n1p2", "259:2");
        fs::write(
            root.path.join("proc/cmdline"),
            "console=ttyS0 root=/dev/nvme0n1p2 rw\n",
        )
        .unwrap();

        let inventory = probe_storage(&root.paths()).unwrap();
        assert_eq!(inventory.root_device_names(), &["nvme0n1p2"]);
        assert_eq!(
            inventory.candidates()[0].blocked_reasons(),
            &[StorageBlockReason::RunningSystemDisk]
        );
    }

    #[test]
    fn storage_probe_blocks_read_only_and_zero_capacity_disks() {
        let root = TestStorageRoot::new();
        root.add_disk("vdb", "252:16", 67_108_864, true);
        root.add_disk("vdc", "252:32", 0, false);

        let inventory = probe_storage(&root.paths()).unwrap();
        let read_only = &inventory.candidates()[0];
        assert_eq!(read_only.device(), "/dev/vdb");
        assert_eq!(read_only.blocked_reasons(), &[StorageBlockReason::ReadOnly]);
        let empty = &inventory.candidates()[1];
        assert_eq!(empty.capacity_bytes(), 0);
        assert_eq!(empty.blocked_reasons(), &[StorageBlockReason::ZeroCapacity]);
        assert_eq!(inventory.eligible_candidates().count(), 0);
    }

    #[test]
    fn storage_probe_excludes_partitions_and_pseudo_devices() {
        let root = TestStorageRoot::new();
        root.add_disk("vda", "252:0", 67_108_864, false);
        root.add_partition("vda", "vda1", "252:1");
        root.add_disk("loop0", "7:0", 1024, false);
        root.add_disk("sr0", "11:0", 1024, true);

        let inventory = probe_storage(&root.paths()).unwrap();
        assert_eq!(inventory.candidates().len(), 1);
        assert_eq!(inventory.candidates()[0].device(), "/dev/vda");
    }

    #[test]
    fn storage_probe_rejects_unbounded_device_enumeration() {
        let root = TestStorageRoot::new();
        for index in 0..=STORAGE_DEVICE_LIMIT {
            fs::create_dir(root.path.join("sys/class/block").join(format!("x{index}"))).unwrap();
        }
        assert_eq!(
            probe_storage(&root.paths()),
            Err(StorageProbeError::TooManyDevices(STORAGE_DEVICE_LIMIT + 1))
        );
    }

    #[test]
    fn storage_probe_rejects_oversized_proc_metadata() {
        let root = TestStorageRoot::new();
        fs::write(
            root.path.join("proc/self/mountinfo"),
            vec![b'x'; STORAGE_METADATA_LIMIT as usize + 1],
        )
        .unwrap();
        assert_eq!(
            probe_storage(&root.paths()),
            Err(StorageProbeError::MetadataTooLarge("proc-mountinfo"))
        );
    }
}
