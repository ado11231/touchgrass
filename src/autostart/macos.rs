//! Manage the macOS launch agent.

use super::{remove_service, run, write_service};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Stable launchd label for the user service.
const LABEL: &str = "com.touchgrass.tracker";

fn paths() -> Result<(PathBuf, PathBuf)> {
    let home = dirs::home_dir().context("could not find home folder")?;
    Ok((
        home.join("Library/LaunchAgents/com.touchgrass.tracker.plist"),
        home.join("Library/Logs/touchgrass.log"),
    ))
}

fn domain() -> Result<String> {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .context("could not read user ID")?;
    ensure!(output.status.success(), "could not read user ID");
    let user_id: u32 = std::str::from_utf8(&output.stdout)?.trim().parse()?;
    Ok(format!("gui/{user_id}"))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn definition(executable: &Path, log: &Path) -> Result<String> {
    let executable = escape(
        executable
            .to_str()
            .context("executable path is not valid UTF-8")?,
    );
    let log = escape(log.to_str().context("log path is not valid UTF-8")?);
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\"><dict>
<key>Label</key><string>{LABEL}</string>
<key>ProgramArguments</key><array><string>{executable}</string><string>track</string></array>
<key>RunAtLoad</key><true/>
<key>KeepAlive</key><true/>
<key>ThrottleInterval</key><integer>10</integer>
<key>StandardOutPath</key><string>{log}</string>
<key>StandardErrorPath</key><string>{log}</string>
</dict></plist>
"))
}

/// Write the login file and start the service.
pub fn install(executable: &Path) -> Result<()> {
    let (path, log) = paths()?;
    fs::create_dir_all(log.parent().context("log path has no parent")?)?;
    let changed = write_service(&path, &definition(executable, &log)?)?;
    let domain = domain()?;
    let service = format!("{domain}/{LABEL}");
    let loaded = Command::new("launchctl")
        .args(["print", &service])
        .output()?
        .status
        .success();
    if loaded && changed {
        run(Command::new("launchctl").args(["bootout", &service]))?;
    }
    if !loaded || changed {
        run(Command::new("launchctl")
            .args(["bootstrap", &domain])
            .arg(&path))?;
    }
    run(Command::new("launchctl").args(["kickstart", &service]))
}

/// Stop the service and remove its login file.
pub fn uninstall() -> Result<()> {
    let (path, _) = paths()?;
    let service = format!("{}/{LABEL}", domain()?);
    if Command::new("launchctl")
        .args(["print", &service])
        .output()?
        .status
        .success()
    {
        run(Command::new("launchctl").args(["bootout", &service]))?;
    }
    remove_service(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_paths_in_plist() -> Result<()> {
        let text = definition(
            Path::new("/tmp/a & b/touchgrass"),
            Path::new("/tmp/a<b.log"),
        )?;
        assert!(text.contains("a &amp; b/touchgrass"));
        assert!(text.contains("a&lt;b.log"));
        assert!(text.contains("<string>track</string>"));
        Ok(())
    }
}
