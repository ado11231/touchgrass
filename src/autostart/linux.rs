//! Manage the Linux systemd user service.

use super::{remove_service, run, write_service};
use anyhow::{Context, Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Stable systemd name for the user service.
const SERVICE: &str = "touchgrass.service";

fn path() -> Result<PathBuf> {
    let config = dirs::config_dir().context("could not find config folder")?;
    Ok(config.join("systemd/user").join(SERVICE))
}

fn definition(executable: &Path) -> Result<String> {
    let executable = executable
        .to_str()
        .context("executable path is not valid UTF-8")?;
    ensure!(
        !executable.chars().any(char::is_control),
        "executable path has control characters"
    );
    let executable = executable
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%")
        .replace('$', "$$");
    Ok(format!(
        "[Unit]
Description=Track active time in dev tools
PartOf=graphical-session.target

[Service]
ExecStart=\"{executable}\" track
Restart=always
RestartSec=5
TimeoutStopSec=15

[Install]
WantedBy=default.target
"
    ))
}

/// Write the login file and start the service.
pub fn install(executable: &Path) -> Result<()> {
    let changed = write_service(&path()?, &definition(executable)?)?;
    run(Command::new("systemctl").args(["--user", "daemon-reload"]))?;
    run(Command::new("systemctl").args(["--user", "enable", "--now", SERVICE]))?;
    if changed {
        run(Command::new("systemctl").args(["--user", "restart", SERVICE]))?;
    }
    Ok(())
}

/// Stop the service and remove its login file.
pub fn uninstall() -> Result<()> {
    let path = path()?;
    let output = Command::new("systemctl")
        .args(["--user", "show", SERVICE, "--property=LoadState", "--value"])
        .output()
        .context("could not query systemd user service")?;
    ensure!(
        output.status.success(),
        "could not query systemd: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if String::from_utf8_lossy(&output.stdout).trim() != "not-found" {
        run(Command::new("systemctl").args(["--user", "disable", "--now", SERVICE]))?;
    }
    remove_service(&path)?;
    run(Command::new("systemctl").args(["--user", "daemon-reload"]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_systemd_paths() -> Result<()> {
        let text = definition(Path::new("/tmp/a b/%x/$name/touchgrass"))?;
        assert!(text.contains("ExecStart=\"/tmp/a b/%%x/$$name/touchgrass\" track"));
        assert!(definition(Path::new("/tmp/bad\npath")).is_err());
        Ok(())
    }
}
