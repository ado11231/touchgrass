//! Read time since the last keyboard or mouse input.

use anyhow::Result;

/// Return idle seconds, or an error when the session cannot report them.
pub fn seconds() -> Result<u64> {
    platform::seconds()
}

#[cfg(target_os = "macos")]
mod platform {
    use anyhow::{Result, ensure};

    /// Read input from all login sessions known to Core Graphics.
    const COMBINED_SESSION: i32 = 0;
    /// Ask for time since any input event.
    const ANY_INPUT: u32 = u32::MAX;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event: u32) -> f64;
    }

    /// Read idle seconds from this platform.
    pub fn seconds() -> Result<u64> {
        // SAFETY: These values are valid Core Graphics event selectors.
        let seconds =
            unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION, ANY_INPUT) };
        ensure!(
            seconds.is_finite() && seconds >= 0.0,
            "could not read system idle time"
        );
        Ok(seconds as u64)
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use anyhow::{Context, Result, bail, ensure};
    use std::{
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };

    /// Limit how long a desktop helper can delay the tracker.
    const HELPER_TIMEOUT: Duration = Duration::from_secs(2);
    /// Check for helper completion without busy waiting.
    const HELPER_POLL: Duration = Duration::from_millis(20);

    /// Read idle seconds from this platform.
    pub fn seconds() -> Result<u64> {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            let output = read(
                "gdbus",
                &[
                    "call",
                    "--session",
                    "--timeout",
                    "1",
                    "--dest",
                    "org.gnome.Mutter.IdleMonitor",
                    "--object-path",
                    "/org/gnome/Mutter/IdleMonitor/Core",
                    "--method",
                    "org.gnome.Mutter.IdleMonitor.GetIdletime",
                ],
            )?;
            return parse_session(&output);
        }
        let output = read("xprintidle", &[])?;
        Ok(output
            .trim()
            .parse::<u64>()
            .context("invalid X11 idle time")?
            / 1000)
    }

    fn read(program: &str, arguments: &[&str]) -> Result<String> {
        let mut child = Command::new(program)
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("could not run {program}"))?;
        let start = Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                let output = child.wait_with_output()?;
                ensure!(
                    output.status.success(),
                    "{program} could not read session idle time"
                );
                return String::from_utf8(output.stdout).context("invalid idle time output");
            }
            if start.elapsed() >= HELPER_TIMEOUT {
                let _ = child.kill();
                let _ = child.wait();
                bail!("{program} took too long to read idle time");
            }
            thread::sleep(HELPER_POLL);
        }
    }

    fn parse_session(output: &str) -> Result<u64> {
        let value = output
            .trim()
            .strip_prefix("(uint64 ")
            .and_then(|value| value.strip_suffix(",)"))
            .context("invalid desktop idle time")?;
        Ok(value.parse::<u64>().context("invalid desktop idle time")? / 1000)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn reads_gnome_response() -> Result<()> {
            assert_eq!(parse_session("(uint64 125000,)\n")?, 125);
            assert!(parse_session("error").is_err());
            Ok(())
        }
    }
}

/// Handle unsupported Linux sessions without stopping time tracking.
pub fn active(limit: u64, warned: &mut bool) -> Result<bool> {
    match seconds() {
        Ok(seconds) => Ok(seconds < limit),
        Err(error) => handle_error(error, warned),
    }
}

#[cfg(target_os = "linux")]
fn handle_error(error: anyhow::Error, warned: &mut bool) -> Result<bool> {
    if !*warned {
        eprintln!("touchgrass: idle detection unavailable; counting as active: {error:#}");
        *warned = true;
    }
    Ok(true)
}

#[cfg(target_os = "macos")]
fn handle_error(error: anyhow::Error, _warned: &mut bool) -> Result<bool> {
    Err(error)
}
