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
            vec![String::from("codex")],
        ];
        let tools = [
            (2, "Terminal", names[0].as_slice()),
            (1, "Editor", names[1].as_slice()),
            (0, "Claude", names[2].as_slice()),
            (0, "Codex", names[3].as_slice()),
        ];
        let processes = vec![
            String::from("kitty"),
            String::from("Code Helper"),
            String::from("node\n/usr/local/CLAUDE/cli.js"),
            String::from("codex"),
        ];
        assert_eq!(pick_tool(tools.into_iter(), &processes), Some("Claude"));
        assert_eq!(
            pick_tool(tools.into_iter(), &processes[..2]),
            Some("Editor")
        );
        assert_eq!(
            pick_tool(tools.into_iter(), &processes[..1]),
            Some("Terminal")
        );
        assert_eq!(pick_tool(tools.into_iter(), &[]), None);
        Ok(())
    }

    #[test]
    fn records_each_side_of_midnight_separately() -> Result<()> {
        let mut days = BTreeMap::new();
        let before = NaiveDate::parse_from_str("2026-12-31", "%Y-%m-%d")?;
        let after = NaiveDate::parse_from_str("2027-01-01", "%Y-%m-%d")?;
        record(&mut days, before, "Codex")?;
        record(&mut days, after, "Codex")?;
        assert_eq!(days[&before]["Codex"], CHECK_INTERVAL.as_secs());
        assert_eq!(days[&after]["Codex"], CHECK_INTERVAL.as_secs());
        Ok(())
    }

    #[test]
    fn rejects_counter_overflow() -> Result<()> {
        let date = NaiveDate::parse_from_str("2026-08-14", "%Y-%m-%d")?;
        let mut days =
            BTreeMap::from([(date, BTreeMap::from([(String::from("Codex"), u64::MAX)]))]);
        assert!(record(&mut days, date, "Codex").is_err());
        assert_eq!(days[&date]["Codex"], u64::MAX);
        Ok(())
    }

    #[test]
    fn lock_is_exclusive_and_released_on_drop() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("touchgrass.lock");
        let lock = acquire_lock(&path)?;
        assert!(acquire_lock(&path).is_err());
        drop(lock);
        let _next = acquire_lock(&path)?;
        Ok(())
    }
    #[test]
    fn sample_schedule_skips_missed_checks() -> Result<()> {
        let previous = Instant::now();
        assert_eq!(next_deadline(previous, previous), previous + CHECK_INTERVAL);
        let after_sleep = previous + SAVE_INTERVAL;
        assert_eq!(
            next_deadline(previous, after_sleep),
            after_sleep + CHECK_INTERVAL
        );
        Ok(())
    }
}
