//! Install and remove the tracker login service.

use anyhow::{Context, Result, ensure};
use std::{fs, io::Write, path::Path, process::Command};

#[cfg(target_os = "macos")]
#[path = "autostart/macos.rs"]
mod platform;
#[cfg(target_os = "linux")]
#[path = "autostart/linux.rs"]
mod platform;

/// Start tracking now and at future logins.
pub fn install(executable: &Path) -> Result<()> {
    platform::install(executable)
}

/// Stop the login service without deleting saved time or settings.
pub fn uninstall() -> Result<()> {
    platform::uninstall()
}

fn run(command: &mut Command) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("could not run {command:?}"))?;
    ensure!(
        output.status.success(),
        "{command:?} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

fn write_service(path: &Path, contents: &str) -> Result<bool> {
    match fs::read_to_string(path) {
        Ok(previous) if previous == contents => return Ok(false),
        Ok(_) => (),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(error).context("could not read login file"),
    }
    let parent = path.parent().context("login file path has no parent")?;
    fs::create_dir_all(parent).context("could not create login folder")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary
        .write_all(contents.as_bytes())
        .context("could not write login file")?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .context("could not replace login file")?;
    Ok(true)
}

fn remove_service(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("could not remove login file"),
    }
}
