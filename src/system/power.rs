use std::process::Command;
use tracing::{error, info};

pub fn reboot(demo_mode: bool) -> std::io::Result<()> {
    if demo_mode {
        info!("Demo mode: skipping reboot");
        return Ok(());
    }
    info!("Executing reboot");
    execute_power_command(&["systemctl", "reboot"])
}

pub fn poweroff(demo_mode: bool) -> std::io::Result<()> {
    if demo_mode {
        info!("Demo mode: skipping poweroff");
        return Ok(());
    }
    info!("Executing poweroff");
    execute_power_command(&["systemctl", "poweroff"])
}

fn execute_power_command(args: &[&str]) -> std::io::Result<()> {
    let status = Command::new(args[0]).args(&args[1..]).status()?;

    if status.success() {
        Ok(())
    } else {
        error!("Power command failed with status: {:?}", status);
        Err(std::io::Error::other("Power command failed"))
    }
}
