use std::sync::Arc;

use lctrl_core::{LctrlError, Platform, Result};
use sailbreak_gui::{DashboardSnapshot, GuiController, run_with_controller};

#[cfg(target_os = "linux")]
fn main() {
    let controller = Arc::new(LinuxController::new());
    let snapshot = snapshot_or_unavailable(controller.as_ref(), Platform::Linux);
    launch(snapshot, controller);
}

#[cfg(windows)]
fn main() {
    let controller = Arc::new(WindowsController::new());
    let snapshot = snapshot_or_unavailable(controller.as_ref(), Platform::Windows);
    launch(snapshot, controller);
}

#[cfg(not(any(target_os = "linux", windows)))]
fn main() {
    let snapshot = DashboardSnapshot::unavailable(
        Platform::Linux,
        "No supported platform HAL is attached; controls remain unavailable",
    );
    launch(snapshot, Arc::new(UnsupportedController));
}

fn launch(snapshot: DashboardSnapshot, controller: Arc<dyn GuiController>) {
    if let Err(error) = run_with_controller(snapshot, controller) {
        exit_error(error);
    }
}

fn snapshot_or_unavailable(
    controller: &dyn GuiController,
    platform: Platform,
) -> DashboardSnapshot {
    match controller.refresh() {
        Ok(snapshot) => snapshot,
        Err(error) => {
            DashboardSnapshot::unavailable(platform, format!("Hardware probe failed: {error}"))
        }
    }
}

fn execute_command(args: &[&str], services: sailbreak_cli::CommandServices<'_>) -> Result<String> {
    if matches!(args, ["daemon", "status"]) {
        let response = sailbreak_daemon::request(&sailbreak_daemon::DaemonRequest::Status)?;
        return serde_json::to_string(&response)
            .map_err(|error| LctrlError::Io(std::io::Error::other(error)));
    }
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("sailbreak");
    argv.extend_from_slice(args);
    let cli =
        sailbreak_cli::Cli::try_parse_from(argv).map_err(|error| LctrlError::InvalidArgument {
            detail: error.to_string(),
        })?;
    Ok(sailbreak_cli::execute_with_services(cli, services)?.human)
}

#[cfg(target_os = "linux")]
struct LinuxController {
    hal: lctrl_hal_linux::LinuxHal,
}

#[cfg(target_os = "linux")]
impl LinuxController {
    fn new() -> Self {
        Self {
            hal: lctrl_hal_linux::LinuxHal::new(),
        }
    }

    fn services(&self) -> sailbreak_cli::CommandServices<'_> {
        sailbreak_cli::CommandServices::new(&self.hal)
            .with_battery(&self.hal)
            .with_conflict_detection(&self.hal)
            .with_diagnostics(&self.hal)
            .with_fan(&self.hal)
            .with_keyboard(&self.hal)
            .with_magicbay(&self.hal)
            .with_privacy(&self.hal)
            .with_performance(&self.hal)
            .with_power(&self.hal)
            .with_touchpad(&self.hal)
            .with_tuning(&self.hal)
            .with_temperature(&self.hal)
            .with_update(&self.hal)
    }
}

#[cfg(target_os = "linux")]
impl GuiController for LinuxController {
    fn refresh(&self) -> Result<DashboardSnapshot> {
        DashboardSnapshot::from_hal(&self.hal)
    }

    fn execute(&self, args: &[&str]) -> Result<String> {
        execute_command(args, self.services())
    }

    fn save_profile(&self, source: &str) -> Result<String> {
        sailbreak_cli::save_user_profile(source)
    }
}

#[cfg(windows)]
struct WindowsController {
    hal: lctrl_hal_win::NativeWindowsHal,
    battery: lctrl_hal_win::WindowsBatteryP0<
        lctrl_hal_win::NativeIoctl,
        lctrl_hal_win::UnverifiedChargeModeReader,
    >,
    conflicts: lctrl_hal_win::WindowsControlConflictDetector<lctrl_hal_win::NativeWmi>,
    bios: lctrl_hal_win::WindowsBiosController<lctrl_hal_win::NativeWmi>,
    performance: lctrl_hal_win::WindowsPerformanceP0<lctrl_hal_win::NativePerformanceRegistry>,
    system_inventory: lctrl_hal_win::WindowsSystemInventory<lctrl_hal_win::NativeWmi>,
    magicbay: lctrl_hal_win::NativeMagicBay,
    peripherals: lctrl_hal_win::WindowsPeripheralController<lctrl_hal_win::NativeWmi>,
    power: lctrl_hal_win::WindowsPowerP0<lctrl_hal_win::NativePowerApi>,
}

#[cfg(windows)]
impl WindowsController {
    fn new() -> Self {
        Self {
            hal: lctrl_hal_win::NativeWindowsHal::new(
                lctrl_hal_win::NativeWmi,
                lctrl_hal_win::NativeIoctl,
            ),
            battery: lctrl_hal_win::WindowsBatteryP0::new(
                lctrl_hal_win::NativeIoctl,
                lctrl_hal_win::UnverifiedChargeModeReader,
            ),
            conflicts: lctrl_hal_win::WindowsControlConflictDetector::new(lctrl_hal_win::NativeWmi),
            bios: lctrl_hal_win::WindowsBiosController::new(lctrl_hal_win::NativeWmi),
            performance: lctrl_hal_win::WindowsPerformanceP0::new(
                lctrl_hal_win::NativePerformanceRegistry,
            ),
            system_inventory: lctrl_hal_win::WindowsSystemInventory::new(lctrl_hal_win::NativeWmi),
            magicbay: lctrl_hal_win::NativeMagicBay,
            peripherals: lctrl_hal_win::WindowsPeripheralController::new(lctrl_hal_win::NativeWmi),
            power: lctrl_hal_win::WindowsPowerP0::new(lctrl_hal_win::NativePowerApi),
        }
    }

    fn services(&self) -> sailbreak_cli::CommandServices<'_> {
        sailbreak_cli::CommandServices::new(&self.hal)
            .with_battery(&self.battery)
            .with_conflict_detection(&self.conflicts)
            .with_bios(&self.bios)
            .with_diagnostics(&self.system_inventory)
            .with_keyboard(&self.peripherals)
            .with_magicbay(&self.magicbay)
            .with_performance(&self.performance)
            .with_power(&self.power)
            .with_panel(&self.peripherals)
            .with_update(&self.system_inventory)
    }
}

#[cfg(windows)]
impl GuiController for WindowsController {
    fn refresh(&self) -> Result<DashboardSnapshot> {
        DashboardSnapshot::from_hal(&self.hal)
    }

    fn execute(&self, args: &[&str]) -> Result<String> {
        execute_command(args, self.services())
    }

    fn save_profile(&self, source: &str) -> Result<String> {
        sailbreak_cli::save_user_profile(source)
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
struct UnsupportedController;

#[cfg(not(any(target_os = "linux", windows)))]
impl GuiController for UnsupportedController {
    fn refresh(&self) -> Result<DashboardSnapshot> {
        Err(LctrlError::Unsupported {
            feature: "gui.platform".into(),
        })
    }

    fn execute(&self, _args: &[&str]) -> Result<String> {
        Err(LctrlError::Unsupported {
            feature: "gui.platform".into(),
        })
    }
}

fn exit_error(error: LctrlError) -> ! {
    eprintln!("{error}");
    std::process::exit(i32::from(error.exit_code()));
}
