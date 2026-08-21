//! Run the CLI with private config and data folders.

use anyhow::{Context, Result, bail, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

/// Give child processes enough time to start or exit on busy machines.
const PROCESS_TIMEOUT: Duration = Duration::from_secs(10);
/// Check files and children without busy waiting.
const POLL_INTERVAL: Duration = Duration::from_millis(25);

/// A private home directory for one test.
pub struct Sandbox {
    directory: TempDir,
}

impl Sandbox {
    /// Create a home without using the real user's files.
    pub fn new() -> Result<Self> {
        Ok(Self {
            directory: tempfile::tempdir()?,
        })
    }

    /// Build a command whose paths are private to this test.
    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_touchgrass"));
        command
            .env("HOME", self.directory.path())
            .env("XDG_CONFIG_HOME", self.directory.path().join("settings"))
            .env("XDG_DATA_HOME", self.directory.path().join("saved"))
            .env("NO_COLOR", "1");
        command
    }

    /// Find a file created by the CLI on either supported platform.
    pub fn find(&self, name: &str) -> Result<PathBuf> {
        find(self.directory.path(), name)?.with_context(|| format!("missing {name}"))
    }

    /// Wait for a tracker to create its first data file.
    pub fn start(&self) -> Result<Running> {
        let previous = self
            .find("data.json")
            .ok()
            .map(fs::metadata)
            .transpose()?
            .map(|metadata| metadata.modified())
            .transpose()?;
        let child = self
            .command()
            .arg("track")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let mut running = Running { child };
        let start = Instant::now();
        loop {
            ensure!(
                running.child.try_wait()?.is_none(),
                "tracker exited before startup"
            );
            if let Ok(path) = self.find("data.json")
                && Some(fs::metadata(path)?.modified()?) != previous
            {
                return Ok(running);
            }
            ensure!(
                start.elapsed() < PROCESS_TIMEOUT,
                "tracker did not create data.json"
            );
            thread::sleep(POLL_INTERVAL);
        }
    }
}

fn find(directory: &Path, name: &str) -> Result<Option<PathBuf>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(path) = find(&entry.path(), name)? {
                return Ok(Some(path));
            }
        } else if entry.file_name() == name {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

/// A child that is always reaped, even if its test fails.
pub struct Running {
    child: Child,
}

impl Running {
    /// Send a shutdown signal and wait for a clean exit.
    pub fn stop(mut self, signal: &str) -> Result<ExitStatus> {
        ensure!(
            Command::new("sh")
                .args([
                    "-c",
                    "kill \"$1\" \"$2\"",
                    "touchgrass-test",
                    signal,
                    &self.child.id().to_string()
                ])
                .status()?
                .success(),
            "could not signal tracker"
        );
        let start = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait()? {
                return Ok(status);
            }
            if start.elapsed() >= PROCESS_TIMEOUT {
                bail!("tracker did not stop");
            }
            thread::sleep(POLL_INTERVAL);
        }
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
