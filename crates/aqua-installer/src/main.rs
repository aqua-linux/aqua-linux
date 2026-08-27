use aqua_installer::{
    build_dry_run_plan, build_install_transaction_graph, compile_install_commands,
    compile_internal_install_actions, probe_storage, revalidate_install_target,
    validate_install_prerequisites, DiskIdentity, InstallArtifacts, InstallCleanupRequirement,
    InstallCommandSpec, InstallMode, InstallPrerequisites, InstallProgressEvent, InstallTarget,
    InstallToolPaths, InstallTransactionGraph, InstallTransactionStep, InstallerFormState,
    InstallerModel, InstallerStep, InstallerUiState, InstallerWindowLayout,
    InternalInstallActionKind, NonExecutingInstallCommandRunner,
    NonExecutingInstallTransactionRunner, NonExecutingInternalInstallRunner, StorageInventory,
    StorageProbePaths, UserProfile, DRY_RUN_PLAN_STATUS, INSTALLER_DISK_FORM_STATUS,
    INSTALLER_FORM_STATUS, INSTALLER_STATUS, INSTALLER_SUMMARY_FORM_STATUS,
    INSTALLER_TIMEZONE_FORM_STATUS, INSTALLER_UI_STATUS, INSTALLER_USER_FORM_STATUS,
    INSTALL_COMMAND_PLAN_STATUS, INSTALL_COMMAND_REHEARSAL_STATUS, INSTALL_PREREQUISITES_STATUS,
    INSTALL_TRANSACTION_GRAPH_STATUS, INSTALL_TRANSACTION_REHEARSAL_STATUS,
    INTERNAL_INSTALL_PLAN_STATUS, INTERNAL_INSTALL_REHEARSAL_STATUS, KEYBOARD_OPTIONS,
    LANGUAGE_OPTIONS, STORAGE_PROBE_STATUS, TIMEZONE_OPTIONS,
};
use aqua_scene::Viewport;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::fs::File;
use std::io::{Read, Write as IoWrite};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const PROBE_STATUS: &str = "recovery-safe-installer-readiness-probe-ready";
const ROOTFS_ARCHIVE: &str = "/run/aqua-installer/rootfs.tar";
const KERNEL_IMAGE: &str = "/run/aqua-installer/bzImage";
const BOOTLOADER_IMAGE: &str = "/run/aqua-installer/bootx64.efi";
const SYNTHETIC_DEVICE: &str = "/dev/aqua-readiness-probe";
const SYNTHETIC_CAPACITY_BYTES: u64 = 32 * 1024 * 1024 * 1024;
const EXECUTION_ENABLE_VALUE: &str = "QEMU_DISPOSABLE_TARGET_ONLY";
const TRANSACTION_EXECUTE_VALUE: &str = "ERASE_DISPOSABLE_VDB_NOW";
const FAILURE_INJECT_AFTER_EFI_MOUNT: &str = "AFTER_EFI_MOUNT";
const EXECUTION_CMDLINE_GATE: &str = "aqua.installer_execution_gate=1";
const QEMU_DISPOSABLE_TARGET: &str = "/dev/vdb";
const INSTALL_COMMAND_TIMEOUT: Duration = Duration::from_secs(180);

fn main() {
    let mut arguments = std::env::args().skip(1);
    let command = arguments.next();
    let stage = match command.as_deref() {
        Some("execution-gate") => "execution-gate",
        Some("execution-run") => "execution-run",
        _ => "readiness-probe",
    };
    let result = match command.as_deref() {
        None | Some("readiness") => run_readiness_probe(),
        Some("execution-gate") => run_execution_gate(arguments.next().as_deref()),
        Some("execution-run") => run_execution_transaction(arguments.next().as_deref()),
        Some("--help" | "-h") => {
            print_help();
            Ok(())
        }
        Some(argument) => Err(format!("unknown argument: {argument}").into()),
    };

    if let Err(error) = result {
        eprintln!("installer_probe_error={error}");
        if stage == "execution-run" && error.to_string().starts_with("transaction failed after ") {
            println!(
                "[AQUA-INSTALLER] stage=execution-run status=error executed=true completed=false"
            );
        } else {
            println!("[AQUA-INSTALLER] stage={stage} status=error executed=false");
        }
        std::process::exit(1);
    }
}

fn print_help() {
    println!(
        "Usage: aqua-installer-probe [readiness|execution-gate CONFIRMATION|execution-run CONFIRMATION]"
    );
    println!("Readiness and execution-gate do not write; execution-run is QEMU /dev/vdb only.");
}

fn run_execution_gate(confirmation: Option<&str>) -> Result<(), Box<dyn Error>> {
    let authorized = authorize_execution(confirmation)?;
    println!("execution_gate_status=authorized-no-execution");
    println!("execution_gate_qemu_runtime=true");
    println!("execution_gate_kernel_cmdline=true");
    println!("execution_gate_operator_enable=true");
    println!("execution_gate_target_revalidated=true");
    println!("execution_gate_artifacts_staged=true");
    println!("execution_gate_artifact_manifest_verified=true");
    println!("execution_gate_confirmation_exact=true");
    println!("execution_gate_expected_confirmation=ERASE /dev/vdb");
    println!(
        "execution_gate_target_device={}",
        authorized.plan.target_device()
    );
    println!(
        "execution_gate_plan_fingerprint={:016x}",
        authorized.plan.fingerprint()
    );
    println!(
        "execution_gate_transaction_steps={}",
        authorized.graph.steps().len()
    );
    println!("install_execution_armed=true");
    println!("transaction_execution_started=false");
    println!("disk_commands_executed=false");
    println!("filesystem_writes_executed=false");
    println!("[AQUA-INSTALLER] stage=execution-gate status=authorized executed=false");
    Ok(())
}

struct AuthorizedExecution {
    model: InstallerModel,
    plan: aqua_installer::InstallPlan,
    graph: InstallTransactionGraph,
}

fn authorize_execution(confirmation: Option<&str>) -> Result<AuthorizedExecution, Box<dyn Error>> {
    let inventory = probe_storage(&StorageProbePaths::system())?;
    let eligible = inventory.eligible_candidates().cloned().collect::<Vec<_>>();
    let [candidate] = eligible.as_slice() else {
        return Err(format!(
            "execution gate requires exactly one eligible target, found {}",
            eligible.len()
        )
        .into());
    };
    let target = candidate.clone().into_erase_target()?;
    if target.disk.device() != QEMU_DISPOSABLE_TARGET {
        return Err("QEMU execution gate accepts only /dev/vdb".into());
    }
    let expected = target.disk.clone();
    let artifacts = InstallArtifacts::new(ROOTFS_ARCHIVE, KERNEL_IMAGE, BOOTLOADER_IMAGE)?;
    validate_staged_artifacts(&artifacts)?;
    let prerequisites = validate_install_prerequisites(&InstallToolPaths::system())?;
    let mut model = installer_model(target, InstallMode::Real)?;
    model.confirm_destructive(confirmation.unwrap_or_default())?;

    let qemu_runtime = qemu_runtime_detected();
    let cmdline_gate = kernel_cmdline_has(EXECUTION_CMDLINE_GATE);
    let operator_enable = std::env::var("AQUA_INSTALLER_QEMU_EXECUTION_ENABLE")
        .is_ok_and(|value| value == EXECUTION_ENABLE_VALUE);
    if !qemu_runtime || !cmdline_gate || !operator_enable {
        return Err("QEMU execution gate is not explicitly enabled".into());
    }

    let revalidated = revalidate_install_target(&StorageProbePaths::system(), &expected)?;
    let plan = build_dry_run_plan(&model, &artifacts)?;
    let commands = compile_install_commands(&plan, &prerequisites)?;
    let internal = compile_internal_install_actions(&plan)?;
    let graph = build_install_transaction_graph(&plan, &commands, &internal)?;
    model.begin_install()?;
    debug_assert_eq!(revalidated.device(), QEMU_DISPOSABLE_TARGET);
    Ok(AuthorizedExecution { model, plan, graph })
}

fn run_execution_transaction(confirmation: Option<&str>) -> Result<(), Box<dyn Error>> {
    let mut authorized = authorize_execution(confirmation)?;
    let execute_enabled = std::env::var("AQUA_INSTALLER_QEMU_TRANSACTION_EXECUTE")
        .is_ok_and(|value| value == TRANSACTION_EXECUTE_VALUE);
    if !execute_enabled {
        return Err("QEMU transaction execution requires the separate exact enable value".into());
    }

    println!("transaction_execution_target=/dev/vdb");
    println!("transaction_execution_started=true");
    println!(
        "transaction_execution_plan_fingerprint={:016x}",
        authorized.plan.fingerprint()
    );
    let inject_after_efi_mount = std::env::var("AQUA_INSTALLER_QEMU_FAILURE_INJECT")
        .is_ok_and(|value| value == FAILURE_INJECT_AFTER_EFI_MOUNT);
    println!(
        "transaction_failure_injection={}",
        if inject_after_efi_mount {
            "after-efi-mount"
        } else {
            "none"
        }
    );
    let report = match execute_qemu_transaction(&authorized.graph, inject_after_efi_mount) {
        Ok(report) => report,
        Err(error) => {
            println!("transaction_execution_completed=false");
            println!("disk_commands_executed=true");
            println!("filesystem_writes_executed=true");
            return Err(error);
        }
    };
    authorized.model.complete_install()?;
    emit_install_progress(&InstallProgressEvent::completed(&authorized.graph)?);
    println!("transaction_execution_completed=true");
    println!("transaction_execution_steps={}", report.completed_steps);
    println!(
        "transaction_execution_commands={}",
        report.completed_commands
    );
    println!(
        "transaction_execution_internal_actions={}",
        report.completed_internal_actions
    );
    println!(
        "transaction_execution_cleanup_commands={}",
        report.cleanup_commands
    );
    println!("disk_commands_executed=true");
    println!("filesystem_writes_executed=true");
    println!("[AQUA-INSTALLER] stage=execution-run status=ok executed=true target=/dev/vdb");
    Ok(())
}

struct TransactionExecutionReport {
    completed_steps: usize,
    completed_commands: usize,
    completed_internal_actions: usize,
    cleanup_commands: usize,
}

fn execute_qemu_transaction(
    graph: &InstallTransactionGraph,
    inject_after_efi_mount: bool,
) -> Result<TransactionExecutionReport, Box<dyn Error>> {
    let mut report = TransactionExecutionReport {
        completed_steps: 0,
        completed_commands: 0,
        completed_internal_actions: 0,
        cleanup_commands: 0,
    };
    let mut root_mounted = false;
    let mut efi_mounted = false;

    emit_install_progress(&InstallProgressEvent::running(graph, 0)?);

    for step in graph.steps() {
        let result: Result<(), Box<dyn Error>> = match step {
            InstallTransactionStep::RevalidateTarget { expected } => {
                revalidate_install_target(&StorageProbePaths::system(), expected)
                    .map(|_| ())
                    .map_err(|error| error.into())
            }
            InstallTransactionStep::Command(command) => {
                let mut result = execute_install_command(command);
                if result.is_ok() && command.operation() == "write-partition-table" {
                    result = wait_for_target_partitions();
                }
                if result.is_ok() {
                    report.completed_commands += 1;
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
                if result.is_ok()
                    && inject_after_efi_mount
                    && command.operation() == "mount-efi-system-partition"
                {
                    result = Err("injected failure after EFI mount".into());
                }
                result
            }
            InstallTransactionStep::Internal(action) => {
                let result = execute_internal_action(action.kind());
                if result.is_ok() {
                    report.completed_internal_actions += 1;
                }
                result
            }
        };
        if let Err(error) = result {
            report.cleanup_commands = execute_transaction_cleanup(graph, efi_mounted, root_mounted);
            emit_install_progress(&InstallProgressEvent::failed(
                graph,
                report.completed_steps,
            )?);
            return Err(format!(
                "transaction failed after {} steps; cleanup_commands={}: {error}",
                report.completed_steps, report.cleanup_commands
            )
            .into());
        }
        report.completed_steps += 1;
        if report.completed_steps < graph.steps().len() {
            emit_install_progress(&InstallProgressEvent::running(
                graph,
                report.completed_steps,
            )?);
        }
    }
    Ok(report)
}

fn emit_install_progress(event: &InstallProgressEvent) {
    println!(
        "[AQUA-INSTALLER-PROGRESS] state={} phase={} operation={} completed={} total={} percent={}",
        event.state().id(),
        event.phase().id(),
        event.operation(),
        event.completed_steps(),
        event.total_steps(),
        event.percent()
    );
}

fn execute_install_command(command: &InstallCommandSpec) -> Result<(), Box<dyn Error>> {
    let stdin = if command.stdin().is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    let mut child = Command::new(command.program())
        .args(command.arguments())
        .current_dir("/")
        .env_clear()
        .env("PATH", "/sbin:/bin:/usr/sbin:/usr/bin")
        .stdin(stdin)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    if let Some(payload) = command.stdin() {
        child
            .stdin
            .take()
            .ok_or("installer command stdin unavailable")?
            .write_all(payload.as_bytes())?;
    }
    let deadline = Instant::now() + INSTALL_COMMAND_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait()? {
            return status.success().then_some(()).ok_or_else(|| {
                format!(
                    "installer command {} failed with {:?}",
                    command.operation(),
                    status.code()
                )
                .into()
            });
        }
        if Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            return Err(format!("installer command {} timed out", command.operation()).into());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_target_partitions() -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if Path::new("/dev/vdb1").exists() && Path::new("/dev/vdb2").exists() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err("partition nodes /dev/vdb1 and /dev/vdb2 did not appear".into())
}

fn execute_internal_action(action: &InternalInstallActionKind) -> Result<(), Box<dyn Error>> {
    match action {
        InternalInstallActionKind::CreateDirectory { path, mode } => {
            ensure_real_directory(path, *mode)
        }
        InternalInstallActionKind::CopyFileAtomic {
            source,
            destination,
            temporary,
            mode,
        } => {
            let metadata = fs::symlink_metadata(source)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(format!("invalid installer copy source {}", source.display()).into());
            }
            let mut reader = File::open(source)?;
            atomic_install_write(destination, temporary, *mode, |writer| {
                std::io::copy(&mut reader, writer).map(|_| ())
            })
        }
        InternalInstallActionKind::WriteFileAtomic {
            destination,
            temporary,
            content,
            mode,
        } => atomic_install_write(destination, temporary, *mode, |writer| {
            writer.write_all(content.as_bytes())
        }),
    }
}

fn ensure_real_directory(path: &Path, mode: u32) -> Result<(), Box<dyn Error>> {
    if !path.starts_with("/mnt/aqua-target")
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!("unsafe installer directory {}", path.display()).into());
    }
    let mut current = std::path::PathBuf::from("/");
    for component in path.components() {
        match component {
            Component::RootDir => continue,
            Component::Normal(component) => current.push(component),
            _ => return Err(format!("unsafe installer path {}", path.display()).into()),
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(format!(
                    "installer path is not a real directory: {}",
                    current.display()
                )
                .into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current)?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn atomic_install_write<F>(
    destination: &Path,
    temporary: &Path,
    mode: u32,
    write: F,
) -> Result<(), Box<dyn Error>>
where
    F: FnOnce(&mut File) -> std::io::Result<()>,
{
    if !destination.starts_with("/mnt/aqua-target")
        || destination.parent() != temporary.parent()
        || destination == temporary
    {
        return Err(format!("unsafe atomic installer target {}", destination.display()).into());
    }
    let parent = destination
        .parent()
        .ok_or("installer target has no parent")?;
    ensure_real_directory(parent, 0o755)?;
    for path in [destination, temporary] {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("installer target is a symlink: {}", path.display()).into());
            }
            Ok(_) if path == temporary => {
                return Err(format!("installer temporary file exists: {}", path.display()).into());
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(format!("installer target is not a file: {}", path.display()).into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temporary)?;
        write(&mut file)?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(mode);
        file.set_permissions(permissions)?;
        file.sync_all()?;
        drop(file);
        fs::rename(temporary, destination)?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result.map_err(Into::into)
}

fn execute_transaction_cleanup(
    graph: &InstallTransactionGraph,
    efi_mounted: bool,
    root_mounted: bool,
) -> usize {
    let mut completed = 0;
    for cleanup in graph.cleanup() {
        let required = match cleanup.requirement() {
            InstallCleanupRequirement::EfiMounted => efi_mounted,
            InstallCleanupRequirement::RootMounted => root_mounted,
        };
        if required {
            let mountpoint = cleanup
                .command()
                .arguments()
                .first()
                .map(String::as_str)
                .unwrap_or("unknown");
            println!("transaction_cleanup_attempt={mountpoint}");
            if execute_install_command(cleanup.command()).is_ok() {
                completed += 1;
                println!("transaction_cleanup_completed={mountpoint}");
            } else {
                println!("transaction_cleanup_failed={mountpoint}");
            }
        }
    }
    completed
}

fn validate_staged_artifacts(artifacts: &InstallArtifacts) -> Result<(), Box<dyn Error>> {
    let expected = [
        ("rootfs.tar", artifacts.rootfs_archive()),
        ("bzImage", artifacts.kernel_image()),
        ("bootx64.efi", artifacts.bootloader_image()),
    ];
    for (name, path) in expected {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| format!("staged {name} artifact {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 {
            return Err(format!("staged {name} artifact is not a non-empty regular file").into());
        }
    }

    let manifest_path = artifacts
        .rootfs_archive()
        .parent()
        .ok_or("staged artifact directory is unavailable")?
        .join("manifest.sha256");
    let metadata = fs::symlink_metadata(&manifest_path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 1024 {
        return Err("artifact manifest is not a bounded regular file".into());
    }
    let manifest = fs::read_to_string(&manifest_path)?;
    let lines = manifest.lines().collect::<Vec<_>>();
    if lines.len() != expected.len() {
        return Err("artifact manifest must contain exactly three entries".into());
    }
    for ((expected_name, path), line) in expected.into_iter().zip(lines) {
        let Some((digest, name)) = line.split_once("  ") else {
            return Err("invalid artifact manifest entry".into());
        };
        if name != expected_name
            || digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || sha256_file(path)? != digest.to_ascii_lowercase()
        {
            return Err(format!("artifact manifest mismatch for {expected_name}").into());
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn qemu_runtime_detected() -> bool {
    [
        "/sys/class/dmi/id/sys_vendor",
        "/sys/class/dmi/id/product_name",
    ]
    .iter()
    .filter_map(|path| fs::read_to_string(path).ok())
    .any(|value| value.to_ascii_lowercase().contains("qemu"))
}

fn kernel_cmdline_has(required: &str) -> bool {
    fs::read_to_string("/proc/cmdline")
        .is_ok_and(|cmdline| cmdline.split_whitespace().any(|value| value == required))
}

fn run_readiness_probe() -> Result<(), Box<dyn Error>> {
    let inventory = probe_storage(&StorageProbePaths::system())?;
    let prerequisites = validate_install_prerequisites(&InstallToolPaths::system())?;
    let report = build_readiness_report(&inventory, &prerequisites)?;
    print!("{report}");
    Ok(())
}

fn build_readiness_report(
    inventory: &StorageInventory,
    prerequisites: &InstallPrerequisites,
) -> Result<String, Box<dyn Error>> {
    let readiness_target = readiness_target(inventory)?;
    let model = readiness_model(readiness_target.target)?;
    let artifacts = InstallArtifacts::new(ROOTFS_ARCHIVE, KERNEL_IMAGE, BOOTLOADER_IMAGE)?;
    let plan = build_dry_run_plan(&model, &artifacts)?;
    let commands = compile_install_commands(&plan, prerequisites)?;
    let internal = compile_internal_install_actions(&plan)?;
    let graph = build_install_transaction_graph(&plan, &commands, &internal)?;
    let ui_layout = InstallerWindowLayout::for_viewport(Viewport::new(1280, 800))?;
    let ui_state = InstallerUiState::new(&model);
    let mut ui_forms = InstallerFormState::default();
    ui_forms.load_storage_inventory(inventory);

    let mut command_runner = NonExecutingInstallCommandRunner::default();
    let command_rehearsal = command_runner.rehearse(&commands);
    let mut internal_runner = NonExecutingInternalInstallRunner::default();
    let internal_rehearsal = internal_runner.rehearse(&internal);
    let transaction_rehearsal = NonExecutingInstallTransactionRunner.rehearse(&graph, None)?;

    let eligible_count = inventory.eligible_candidates().count();
    let mut report = String::new();
    writeln!(report, "product=Aqua Linux")?;
    writeln!(report, "probe_status={PROBE_STATUS}")?;
    writeln!(report, "state_model_status={INSTALLER_STATUS}")?;
    writeln!(report, "installer_ui_status={INSTALLER_UI_STATUS}")?;
    writeln!(report, "installer_ui_viewport=1280x800")?;
    writeln!(
        report,
        "installer_ui_window={},{},{},{}",
        ui_layout.window.x, ui_layout.window.y, ui_layout.window.width, ui_layout.window.height
    )?;
    writeln!(
        report,
        "installer_ui_step_count={}",
        InstallerStep::ALL.len()
    )?;
    writeln!(report, "installer_ui_focus={}", ui_state.focus().id())?;
    writeln!(report, "installer_ui_keyboard_navigation=true")?;
    writeln!(report, "installer_form_status={INSTALLER_FORM_STATUS}")?;
    writeln!(
        report,
        "installer_language_option_count={}",
        LANGUAGE_OPTIONS.len()
    )?;
    writeln!(
        report,
        "installer_keyboard_option_count={}",
        KEYBOARD_OPTIONS.len()
    )?;
    writeln!(
        report,
        "installer_timezone_form_status={INSTALLER_TIMEZONE_FORM_STATUS}"
    )?;
    writeln!(
        report,
        "installer_timezone_option_count={}",
        TIMEZONE_OPTIONS.len()
    )?;
    writeln!(
        report,
        "installer_user_form_status={INSTALLER_USER_FORM_STATUS}"
    )?;
    writeln!(report, "installer_user_password_content_stored=false")?;
    writeln!(
        report,
        "installer_summary_form_status={INSTALLER_SUMMARY_FORM_STATUS}"
    )?;
    writeln!(report, "installer_summary_target_bound=true")?;
    writeln!(
        report,
        "installer_disk_form_status={INSTALLER_DISK_FORM_STATUS}"
    )?;
    writeln!(
        report,
        "installer_disk_option_count={}",
        ui_forms.disk_options().len()
    )?;
    writeln!(
        report,
        "installer_disk_eligible_count={}",
        ui_forms
            .disk_options()
            .iter()
            .filter(|option| option.is_eligible())
            .count()
    )?;
    writeln!(report, "installer_ui_rendered=false")?;
    writeln!(report, "storage_probe_status={STORAGE_PROBE_STATUS}")?;
    writeln!(
        report,
        "prerequisites_status={INSTALL_PREREQUISITES_STATUS}"
    )?;
    writeln!(report, "dry_run_plan_status={DRY_RUN_PLAN_STATUS}")?;
    writeln!(report, "command_plan_status={INSTALL_COMMAND_PLAN_STATUS}")?;
    writeln!(
        report,
        "command_rehearsal_status={INSTALL_COMMAND_REHEARSAL_STATUS}"
    )?;
    writeln!(
        report,
        "internal_plan_status={INTERNAL_INSTALL_PLAN_STATUS}"
    )?;
    writeln!(
        report,
        "internal_rehearsal_status={INTERNAL_INSTALL_REHEARSAL_STATUS}"
    )?;
    writeln!(
        report,
        "transaction_graph_status={INSTALL_TRANSACTION_GRAPH_STATUS}"
    )?;
    writeln!(
        report,
        "transaction_rehearsal_status={INSTALL_TRANSACTION_REHEARSAL_STATUS}"
    )?;
    writeln!(
        report,
        "storage_candidate_count={}",
        inventory.candidates().len()
    )?;
    writeln!(report, "storage_eligible_count={eligible_count}")?;
    for (index, candidate) in inventory.candidates().iter().enumerate() {
        let reasons = candidate
            .blocked_reasons()
            .iter()
            .map(|reason| reason.id())
            .collect::<Vec<_>>()
            .join(",");
        writeln!(
            report,
            "storage.{index:02}=device:{} capacity_bytes:{} removable:{} eligible:{} blocked:{}",
            candidate.device(),
            candidate.capacity_bytes(),
            candidate.removable(),
            candidate.is_eligible(),
            if reasons.is_empty() { "none" } else { &reasons }
        )?;
    }
    writeln!(
        report,
        "validated_tool_count={}",
        prerequisites.tools().len()
    )?;
    writeln!(
        report,
        "readiness_target_source={}",
        readiness_target.source
    )?;
    writeln!(
        report,
        "readiness_target_device={}",
        readiness_target.device
    )?;
    writeln!(report, "readiness_target_bound=true")?;
    writeln!(report, "readiness_target_selected_for_install=false")?;
    writeln!(report, "install_execution_armed=false")?;
    writeln!(
        report,
        "artifact_rootfs_present={}",
        Path::new(ROOTFS_ARCHIVE).is_file()
    )?;
    writeln!(
        report,
        "artifact_kernel_present={}",
        Path::new(KERNEL_IMAGE).is_file()
    )?;
    writeln!(
        report,
        "artifact_bootloader_present={}",
        Path::new(BOOTLOADER_IMAGE).is_file()
    )?;
    writeln!(report, "plan_ready=true")?;
    writeln!(report, "plan_target_device={}", plan.target_device())?;
    writeln!(report, "plan_operation_count={}", plan.operations().len())?;
    writeln!(report, "plan_fingerprint={:016x}", plan.fingerprint())?;
    writeln!(report, "command_plan_ready=true")?;
    writeln!(
        report,
        "command_count={}",
        command_rehearsal.command_count()
    )?;
    writeln!(report, "internal_plan_ready=true")?;
    writeln!(
        report,
        "internal_action_count={}",
        internal_rehearsal.action_count()
    )?;
    writeln!(report, "transaction_graph_ready=true")?;
    writeln!(
        report,
        "transaction_step_count={}",
        transaction_rehearsal.rehearsed_steps().len()
    )?;
    writeln!(
        report,
        "transaction_cleanup_count={}",
        graph.cleanup().len()
    )?;
    writeln!(report, "execution_allowed=false")?;
    writeln!(report, "disk_commands_executed=false")?;
    writeln!(report, "filesystem_writes_executed=false")?;
    writeln!(report, "recovery_safe=true")?;
    writeln!(
        report,
        "[AQUA-INSTALLER] stage=readiness-probe status=ok executed=false"
    )?;
    Ok(report)
}

struct ReadinessTarget {
    source: &'static str,
    device: String,
    target: InstallTarget,
}

fn readiness_target(inventory: &StorageInventory) -> Result<ReadinessTarget, Box<dyn Error>> {
    let eligible = inventory.eligible_candidates().cloned().collect::<Vec<_>>();
    if let [candidate] = eligible.as_slice() {
        let device = candidate.device().to_string();
        return Ok(ReadinessTarget {
            source: "storage-probe",
            device,
            target: candidate.clone().into_erase_target()?,
        });
    }

    let disk = DiskIdentity::new(
        SYNTHETIC_DEVICE,
        "aqua-synthetic-readiness-target",
        "Aqua non-executing readiness target",
        SYNTHETIC_CAPACITY_BYTES,
    )?;
    Ok(ReadinessTarget {
        source: "synthetic-readiness",
        device: SYNTHETIC_DEVICE.to_string(),
        target: InstallTarget::erase_disk(disk),
    })
}

fn readiness_model(target: InstallTarget) -> Result<InstallerModel, Box<dyn Error>> {
    installer_model(target, InstallMode::DryRun)
}

fn installer_model(
    target: InstallTarget,
    mode: InstallMode,
) -> Result<InstallerModel, Box<dyn Error>> {
    let mut model = InstallerModel::default();
    model.set_mode(mode);
    model.advance()?;
    model.set_locale("tr_TR.UTF-8")?;
    model.advance()?;
    model.set_keyboard_layout("tr")?;
    model.advance()?;
    model.set_target(target);
    model.advance()?;
    model.set_timezone("Europe/Istanbul")?;
    model.advance()?;
    model.set_user(UserProfile::new("aqua", "Aqua User", true)?);
    let step = model.advance()?;
    if step != InstallerStep::Summary {
        return Err("readiness model did not reach summary".into());
    }
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let sequence = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "aqua-installer-probe-{}-{nonce}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn tool(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        fn inventory(&self) -> StorageInventory {
            let sys_class_block = self.0.join("sys/class/block");
            fs::create_dir_all(&sys_class_block).unwrap();
            let proc_mountinfo = self.0.join("proc-mountinfo");
            let proc_cmdline = self.0.join("proc-cmdline");
            fs::write(&proc_mountinfo, "").unwrap();
            fs::write(&proc_cmdline, "root=/dev/vda rw\n").unwrap();
            probe_storage(&StorageProbePaths {
                sys_class_block,
                proc_mountinfo,
                proc_cmdline,
            })
            .unwrap()
        }

        fn add_disk(&self, name: &str, major_minor: &str, capacity_bytes: u64) {
            let sys_class_block = self.0.join("sys/class/block");
            let device = self.0.join("devices").join(name);
            fs::create_dir_all(device.join("device")).unwrap();
            fs::create_dir_all(&sys_class_block).unwrap();
            fs::write(device.join("size"), format!("{}\n", capacity_bytes / 512)).unwrap();
            fs::write(device.join("dev"), format!("{major_minor}\n")).unwrap();
            fs::write(device.join("ro"), "0\n").unwrap();
            fs::write(device.join("removable"), "0\n").unwrap();
            fs::write(device.join("uevent"), "DEVTYPE=disk\n").unwrap();
            fs::write(device.join("device/model"), "QEMU HARDDISK\n").unwrap();
            fs::write(device.join("device/serial"), format!("aqua-{name}\n")).unwrap();
            symlink(device, sys_class_block.join(name)).unwrap();
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn readiness_model_builds_a_non_executable_pipeline() {
        let root = TestRoot::new();
        let prerequisites = validate_install_prerequisites(&InstallToolPaths {
            sfdisk: root.tool("sfdisk"),
            mkfs_fat: root.tool("mkfs.fat"),
            mkfs_ext4: root.tool("mkfs.ext4"),
            tar: root.tool("tar"),
            mount: root.tool("mount"),
            umount: root.tool("umount"),
        })
        .unwrap();
        let inventory = root.inventory();
        let report = build_readiness_report(&inventory, &prerequisites).unwrap();

        assert!(report.contains("readiness_target_source=synthetic-readiness"));
        assert!(report.contains("plan_operation_count=13"));
        assert!(report.contains("command_count=8"));
        assert!(report.contains("internal_action_count=11"));
        assert!(report.contains("transaction_step_count=20"));
        assert!(report
            .contains("installer_ui_status=keyboard-navigable-installer-window-contract-ready"));
        assert!(report.contains("installer_ui_viewport=1280x800"));
        assert!(report.contains("installer_ui_window=32,32,1216,736"));
        assert!(report.contains("installer_ui_step_count=9"));
        assert!(report.contains("installer_ui_focus=language-control"));
        assert!(report.contains("installer_ui_keyboard_navigation=true"));
        assert!(report
            .contains("installer_form_status=validated-language-keyboard-form-controls-ready"));
        assert!(report.contains("installer_language_option_count=3"));
        assert!(report.contains("installer_keyboard_option_count=3"));
        assert!(
            report.contains("installer_timezone_form_status=validated-timezone-form-control-ready")
        );
        assert!(report.contains("installer_timezone_option_count=4"));
        assert!(report.contains("installer_user_form_status=password-content-free-user-form-ready"));
        assert!(report.contains("installer_user_password_content_stored=false"));
        assert!(report
            .contains("installer_summary_form_status=target-bound-summary-confirmation-ready"));
        assert!(report.contains("installer_summary_target_bound=true"));
        assert!(report.contains("installer_disk_form_status=eligible-storage-selection-form-ready"));
        assert!(report.contains("installer_disk_option_count=0"));
        assert!(report.contains("installer_disk_eligible_count=0"));
        assert!(report.contains("installer_ui_rendered=false"));
        assert!(report.contains("execution_allowed=false"));
        assert!(report.contains("disk_commands_executed=false"));
        assert!(report.contains("stage=readiness-probe status=ok executed=false"));
    }

    #[test]
    fn exactly_one_eligible_disk_binds_only_the_readiness_plan() {
        let root = TestRoot::new();
        root.add_disk("vdb", "252:16", 4 * 1024 * 1024 * 1024);
        let prerequisites = validate_install_prerequisites(&InstallToolPaths {
            sfdisk: root.tool("sfdisk"),
            mkfs_fat: root.tool("mkfs.fat"),
            mkfs_ext4: root.tool("mkfs.ext4"),
            tar: root.tool("tar"),
            mount: root.tool("mount"),
            umount: root.tool("umount"),
        })
        .unwrap();
        let report = build_readiness_report(&root.inventory(), &prerequisites).unwrap();

        assert!(report.contains("storage_eligible_count=1"));
        assert!(report.contains("readiness_target_source=storage-probe"));
        assert!(report.contains("readiness_target_device=/dev/vdb"));
        assert!(report.contains("readiness_target_bound=true"));
        assert!(report.contains("readiness_target_selected_for_install=false"));
        assert!(report.contains("install_execution_armed=false"));
        assert!(report.contains("disk_commands_executed=false"));
    }

    #[test]
    fn staged_artifacts_must_be_non_empty_regular_files() {
        let root = TestRoot::new();
        let rootfs = root.0.join("rootfs.tar");
        let kernel = root.0.join("bzImage");
        let bootloader = root.0.join("bootx64.efi");
        fs::write(&rootfs, "rootfs").unwrap();
        fs::write(&kernel, "kernel").unwrap();
        fs::write(&bootloader, "bootloader").unwrap();
        fs::write(
            root.0.join("manifest.sha256"),
            format!(
                "{}  rootfs.tar\n{}  bzImage\n{}  bootx64.efi\n",
                sha256_file(&rootfs).unwrap(),
                sha256_file(&kernel).unwrap(),
                sha256_file(&bootloader).unwrap()
            ),
        )
        .unwrap();
        let artifacts = InstallArtifacts::new(&rootfs, &kernel, &bootloader).unwrap();

        validate_staged_artifacts(&artifacts).unwrap();
        fs::write(&kernel, "").unwrap();
        assert!(validate_staged_artifacts(&artifacts).is_err());
    }
}
