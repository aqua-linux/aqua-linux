use aqua_shell::{
    MotionEasing, MotionValue, SemanticMotion, ShellMotionController, ShellMotionSurface,
};

fn main() {
    println!("revision=aqua-motion-fixtures-1");
    for role in [
        SemanticMotion::Feedback,
        SemanticMotion::Panel,
        SemanticMotion::Menu,
        SemanticMotion::Window,
        SemanticMotion::Workspace,
        SemanticMotion::Notification,
        SemanticMotion::Progress,
        SemanticMotion::Attention,
    ] {
        println!(
            "token role={role:?} duration_ms={} spatial={} repeating={}",
            role.duration_ms(),
            role.is_spatial(),
            role.repeats()
        );
    }
    for easing in [
        MotionEasing::Standard,
        MotionEasing::Enter,
        MotionEasing::Exit,
    ] {
        let points = easing.control_points();
        println!(
            "easing role={easing:?} points={:.1},{:.1},{:.1},{:.1}",
            points[0], points[1], points[2], points[3]
        );
    }

    let mut panel = MotionValue::new(0.0);
    panel.retarget(0, 1.0, SemanticMotion::Panel, false);
    for now_ms in [0, 50, 100, 150, 200] {
        let sample = panel.sample(now_ms);
        println!(
            "panel time_ms={now_ms} value={:.4} active={}",
            sample.value, sample.active
        );
    }

    let mut interrupted = MotionValue::new(0.0);
    interrupted.retarget(0, 1.0, SemanticMotion::Panel, false);
    let before = interrupted.sample(80).value;
    let retarget = interrupted
        .retarget(80, 0.0, SemanticMotion::Panel, false)
        .value;
    let reverse = interrupted.sample(120).value;
    let before_second = interrupted.sample(140).value;
    let second_retarget = interrupted
        .retarget(140, 1.0, SemanticMotion::Panel, false)
        .value;
    println!(
        "interruption before={before:.4} retarget={retarget:.4} reverse={reverse:.4} second_before={before_second:.4} second_retarget={second_retarget:.4} continuous={}",
        (before - retarget).abs() < 0.0001 && (before_second - second_retarget).abs() < 0.0001
    );

    for frame_ms in [17_u64, 11, 8, 7] {
        let mut value = MotionValue::new(0.0);
        value.retarget(1_000, 1.0, SemanticMotion::Panel, false);
        let mut now_ms = 1_000;
        while now_ms < 1_100 {
            now_ms = (now_ms + frame_ms).min(1_100);
            value.sample(now_ms);
        }
        println!(
            "cadence frame_ms={frame_ms} sample_time_ms=1100 value={:.4}",
            value.rendered()
        );
    }

    let mut reduced = ShellMotionController::default();
    reduced.set_reduced_motion(0, true);
    reduced.set_visible(ShellMotionSurface::Launcher, true, 10);
    reduced.set_visible(ShellMotionSurface::SessionMenu, true, 10);
    reduced.set_visible(ShellMotionSurface::Notification, true, 10);
    let frame = reduced.sample(10);
    println!(
        "reduced active={} launcher_opacity={:.1} launcher_offset_y={} menu_offset_y={} notification_offset_x={} repeating_allowed=false state_feedback=true",
        frame.is_active(),
        frame.launcher.opacity,
        frame.launcher.offset_y,
        frame.session_menu.offset_y,
        frame.notification.offset_x,
    );
}
