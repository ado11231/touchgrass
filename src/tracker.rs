//! Match tools and count active time at fixed intervals.

use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions, TryLockError},
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Time credited for one active sample.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(5);
/// Save time and reload settings at this interval.
pub const SAVE_INTERVAL: Duration = Duration::from_secs(60);
/// Stop counting after this long without input unless configured otherwise.
pub const DEFAULT_IDLE_AFTER: u64 = 120;

/// Work requested by the tracker timer.
pub enum Event {
    /// Check activity and credit one tool.
    Sample,
    /// Save time and reload the config.
    Maintain,
    /// Save before leaving the loop.
    Stop,
}

/// Hold the OS lock until tracking stops.
pub fn acquire_lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .context("could not open touchgrass.lock")?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => bail!("another touchgrass tracker is already running"),
        Err(error) => Err(error).context("could not lock touchgrass.lock"),
    }
}

/// Read process names and full command lines from a reusable snapshot.
pub fn processes(system: &mut System) -> Vec<String> {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always),
    );
    system
        .processes()
        .values()
        .map(|process| {
            let mut text = process.name().to_string_lossy().into_owned();
            for argument in process.cmd() {
                text.push('\n');
                text.push_str(&argument.to_string_lossy());
            }
            text
        })
        .collect()
}

/// Pick the first matching tool in the highest priority group.
pub fn pick_tool<'a>(
    tools: impl Iterator<Item = (usize, &'a str, &'a [String])>,
    processes: &[String],
) -> Option<&'a str> {
    let processes: Vec<_> = processes
        .iter()
        .map(|process| process.to_lowercase())
        .collect();
    tools
        .filter(|(_, _, names)| {
            names.iter().any(|name| {
                let needle = name.to_lowercase();
                processes.iter().any(|process| process.contains(&needle))
            })
        })
        .min_by_key(|(priority, _, _)| *priority)
        .map(|(_, name, _)| name)
}

/// Credit one sample to the date on which it was observed.
pub fn record(
    days: &mut BTreeMap<NaiveDate, BTreeMap<String, u64>>,
    date: NaiveDate,
    tool: &str,
) -> Result<()> {
    let totals = days.entry(date).or_default();
    if let Some(seconds) = totals.get_mut(tool) {
        *seconds = seconds
            .checked_add(CHECK_INTERVAL.as_secs())
            .context("saved time is too large")?;
    } else {
        totals.insert(String::from(tool), CHECK_INTERVAL.as_secs());
    }
    Ok(())
}

/// Install shutdown handling before the first saved file is written.
pub fn shutdown_signal() -> Result<mpsc::Receiver<()>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    ctrlc::set_handler(move || {
        let _ = sender.try_send(());
    })
    .context("could not install shutdown handler")?;
    Ok(receiver)
}

/// Run until a termination signal arrives, then request a final save.
pub fn run(
    receiver: mpsc::Receiver<()>,
    mut handle: impl FnMut(Event) -> Result<()>,
) -> Result<()> {
    let mut last_save = Instant::now();
    let mut next_check = Instant::now() + CHECK_INTERVAL;
    loop {
        match receiver.recv_timeout(next_check.saturating_duration_since(Instant::now())) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return handle(Event::Stop),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Err(error) = handle(Event::Sample) {
                    handle(Event::Stop).context("could not save after a tracking error")?;
                    return Err(error);
                }
                if last_save.elapsed() >= SAVE_INTERVAL {
                    handle(Event::Maintain)?;
                    last_save = Instant::now();
                }
                next_check = next_deadline(next_check, Instant::now());
            }
        }
    }
}

fn next_deadline(previous: Instant, now: Instant) -> Instant {
    let next = previous + CHECK_INTERVAL;
    if next <= now {
        // Skip missed checks after sleep instead of counting them all at once.
        now + CHECK_INTERVAL
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_beats_list_order_and_ties_keep_first() -> Result<()> {
        let names = [
            vec![String::from("kitty")],
            vec![String::from("code")],
            vec![String::from("claude")],
