//! Check commands without touching the user's saved time.

#[path = "support/helpers.rs"]
mod helpers;

use anyhow::Result;
use helpers::Sandbox;
use std::fs;

#[test]
fn help_and_bad_commands_do_not_create_config() -> Result<()> {
    let sandbox = Sandbox::new()?;
    let output = sandbox.command().arg("help").output()?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("touchgrass install"));
    for arguments in [
        vec!["unknown"],
        vec!["0"],
        vec!["10000"],
        vec!["track", "extra"],
    ] {
        let output = sandbox.command().args(arguments).output()?;
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
    }
    assert!(sandbox.find("config.toml").is_err());
    Ok(())
}

#[test]
fn missing_data_shows_zeros_and_creates_only_config() -> Result<()> {
    let sandbox = Sandbox::new()?;
    let output = sandbox.command().output()?;
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout)?;
    assert!(text.contains("0h 00m"));
    assert!(text.contains("Run touchgrass install"));
    assert!(!text.contains('\x1b'));
    assert!(sandbox.find("config.toml").is_ok());
    assert!(sandbox.find("data.json").is_err());
    Ok(())
}

#[test]
fn year_view_reads_saved_time_and_reports_bad_config() -> Result<()> {
    let sandbox = Sandbox::new()?;
    assert!(sandbox.start()?.stop("-TERM")?.success());
    let path = sandbox.find("data.json")?;
    let saved = br#"{"days":{"2024-02-29":{"Codex":3600}}}"#;
    fs::write(&path, saved)?;
    fs::write(sandbox.find("config.toml")?, "bad = [")?;
    let output = sandbox.command().arg("2024").output()?;
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout)?;
    let mut lines = text.lines().skip_while(|line| !line.contains("2024 TOTAL"));
    assert!(
        lines
            .nth(1)
            .is_some_and(|values| values.trim_start().starts_with("1h 00m"))
    );
    assert!(String::from_utf8(output.stderr)?.contains("config.toml"));
    assert_eq!(fs::read(&path)?, saved);
    Ok(())
}

#[test]
fn dashboard_reports_corruption_without_changing_files() -> Result<()> {
    let sandbox = Sandbox::new()?;
    assert!(sandbox.start()?.stop("-TERM")?.success());
    let path = sandbox.find("data.json")?;
    fs::write(&path, b"bad json")?;
    let output = sandbox.command().output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("preserve"));
    assert_eq!(fs::read(&path)?, b"bad json");
    assert!(sandbox.find("data.json.bak").is_err());
    Ok(())
}

#[test]
fn install_and_uninstall_are_repeatable_and_keep_data() -> Result<()> {
    use std::{env, os::unix::fs::PermissionsExt};

    let sandbox = Sandbox::new()?;
    assert!(sandbox.start()?.stop("-TERM")?.success());
    let data_path = sandbox.find("data.json")?;
    let config_path = sandbox.find("config.toml")?;
    let data = fs::read(&data_path)?;
    let config = fs::read(&config_path)?;
    let helpers = tempfile::tempdir()?;
    let state = helpers.path().join("running");
    let script = r#"#!/bin/sh
set -eu
case "$1" in
  print|kickstart) test -f "$TOUCHGRASS_SERVICE_STATE" ;;
  bootstrap) touch "$TOUCHGRASS_SERVICE_STATE" ;;
  bootout) rm -f "$TOUCHGRASS_SERVICE_STATE" ;;
  --user)
    case "$2" in
      daemon-reload) : ;;
      enable|restart) touch "$TOUCHGRASS_SERVICE_STATE" ;;
      disable) rm -f "$TOUCHGRASS_SERVICE_STATE" ;;
      show)
        if test -f "$TOUCHGRASS_SERVICE_STATE"; then
          printf 'loaded\n'
        else
          printf 'not-found\n'
        fi ;;
      *) exit 1 ;;
    esac ;;
  *) exit 1 ;;
esac
"#;
    for name in ["launchctl", "systemctl"] {
        let path = helpers.path().join(name);
        fs::write(&path, script)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    let search_path = env::var_os("PATH").ok_or_else(|| anyhow::anyhow!("PATH is missing"))?;
    let search_path = env::join_paths(
        std::iter::once(helpers.path().to_path_buf()).chain(env::split_paths(&search_path)),
    )?;
    let run = |argument| -> Result<()> {
        let output = sandbox
            .command()
            .arg(argument)
            .env("PATH", &search_path)
            .env("TOUCHGRASS_SERVICE_STATE", &state)
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    };
    run("install")?;
    let service = sandbox
        .find("com.touchgrass.tracker.plist")
        .or_else(|_| sandbox.find("touchgrass.service"))?;
    let definition = fs::read(&service)?;
    run("install")?;
    assert_eq!(fs::read(&service)?, definition);
    assert!(state.exists());
    run("uninstall")?;
    run("uninstall")?;
    assert!(!state.exists());
    assert!(!service.exists());
    assert_eq!(fs::read(data_path)?, data);
    assert_eq!(fs::read(config_path)?, config);
    Ok(())
}
