//! Check tracker locking, recovery, and shutdown saves.

#[path = "support/helpers.rs"]
mod helpers;

use anyhow::Result;
use helpers::Sandbox;
use std::{fs, thread, time::Duration};

/// Leave room for one sample and process scheduling before shutdown.
const SAMPLE_WAIT: Duration = Duration::from_secs(8);

#[test]
fn second_tracker_exits_and_shutdown_releases_lock() -> Result<()> {
    let sandbox = Sandbox::new()?;
    let first = sandbox.start()?;
    let output = sandbox.command().arg("track").output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("already running"));
    assert!(first.stop("-TERM")?.success());
    assert!(sandbox.start()?.stop("-INT")?.success());
    Ok(())
}

#[test]
fn shutdown_saves_time_for_both_signals() -> Result<()> {
    for signal in ["-INT", "-TERM"] {
        let sandbox = Sandbox::new()?;
        assert!(sandbox.command().output()?.status.success());
        fs::write(
            sandbox.find("config.toml")?,
            "idle_after = 9223372036854775807\n[[tools]]\nname = 'Test tool'\nkind = 'ai'\nnames = ['touchgrass']\n",
        )?;
        let running = sandbox.start()?;
        thread::sleep(SAMPLE_WAIT);
        assert!(running.stop(signal)?.success());
        let data: serde_json::Value =
            serde_json::from_slice(&fs::read(sandbox.find("data.json")?)?)?;
        assert!(
            data["days"]
                .as_object()
                .is_some_and(|days| days.values().any(|tools| tools["Test tool"]
                    .as_u64()
                    .is_some_and(|seconds| seconds > 0)))
        );
    }
    Ok(())
}

#[test]
fn tracker_preserves_corruption_before_restarting() -> Result<()> {
    let sandbox = Sandbox::new()?;
    assert!(sandbox.start()?.stop("-TERM")?.success());
    let path = sandbox.find("data.json")?;
    fs::write(&path, b"broken data")?;
    let running = sandbox.start()?;
    assert!(running.stop("-TERM")?.success());
    assert_eq!(fs::read(sandbox.find("data.json.bak")?)?, b"broken data");
    let data: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    assert!(data["installed_on"].is_string());
    Ok(())
}
