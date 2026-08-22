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

/// One running program and where it sits in the process tree.
#[derive(Clone)]
pub struct Process {
    /// Process ID.
    pub pid: u32,
    /// Parent process ID, when known.
    pub parent: Option<u32>,
    /// Program name followed by each argument on its own line.
    pub text: String,
}

/// Read process names and full command lines from a reusable snapshot.
pub fn processes(system: &mut System) -> Vec<Process> {
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
            Process {
                pid: process.pid().as_u32(),
                parent: process.parent().map(sysinfo::Pid::as_u32),
                text,
            }
        })
        .collect()
}

/// Check whether a program name or argument names this tool.
///
/// The name must start a path component and end at a non-alphanumeric
/// character, so `code` matches `Code Helper` but not `codex` or `claude-code`.
fn matches(text: &str, needle: &str) -> bool {
    let text = text.to_lowercase();
    let needle = needle.to_lowercase();
    text.split(['\n', '/']).any(|part| {
        part.strip_prefix(needle.as_str())
            .is_some_and(|rest| !rest.starts_with(char::is_alphanumeric))
    })
}

/// Pick the first matching tool in the highest priority group.
pub fn pick_tool<'a>(
    tools: impl Iterator<Item = (usize, &'a str, &'a [String])>,
    processes: &[&str],
) -> Option<&'a str> {
    tools
        .filter(|(_, _, names)| {
            names
                .iter()
                .any(|name| processes.iter().any(|process| matches(process, name)))
        })
        .min_by_key(|(priority, _, _)| *priority)
        .map(|(_, name, _)| name)
}

/// Pick the tools in use right now.
///
/// With a focused window, credit the focused app's own tool and the best tool
/// running inside it, such as Claude Code in a kitty tab. Untracked apps get
/// nothing, even when a tool runs inside them, such as a browser helper.
/// Without a focused window, fall back to the best tool running anywhere.
pub fn pick_tools<'a>(
    tools: &[(usize, &'a str, &'a [String])],
    processes: &[Process],
    focused: Option<u32>,
) -> Vec<&'a str> {
    let Some(root) = focused.filter(|pid| processes.iter().any(|process| process.pid == *pid))
    else {
        let all: Vec<_> = processes
            .iter()
            .map(|process| process.text.as_str())
            .collect();
        return pick_tool(tools.iter().copied(), &all).into_iter().collect();
    };
    let mut tree = vec![root];
    let mut index = 0;
    while let Some(parent) = tree.get(index).copied() {
        tree.extend(
            processes
                .iter()
                .filter(|process| process.parent == Some(parent) && process.pid != parent)
                .map(|process| process.pid),
        );
        index += 1;
    }
    let text = |pid: u32| {
        processes
            .iter()
            .filter(move |process| process.pid == pid)
            .map(|process| process.text.as_str())
    };
    let own: Vec<_> = text(root).collect();
    let inner: Vec<_> = tree.iter().skip(1).flat_map(|pid| text(*pid)).collect();
    let Some(app) = pick_tool(tools.iter().copied(), &own) else {
        return Vec::new();
    };
    let inside = pick_tool(
        tools.iter().copied().filter(|(_, name, _)| *name != app),
        &inner,
    );
    std::iter::once(app).chain(inside).collect()
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

/// Credit one sample of wall-clock time, however many tools were in use.
pub fn record_active(active: &mut BTreeMap<NaiveDate, u64>, date: NaiveDate) -> Result<()> {
    let seconds = active.entry(date).or_default();
    *seconds = seconds
        .checked_add(CHECK_INTERVAL.as_secs())
        .context("saved time is too large")?;
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
        let processes = [
            "kitty",
            "Code Helper",
            "node\n/usr/local/CLAUDE/cli.js",
            "codex",
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
    fn names_match_whole_words_at_path_starts() {
        assert!(matches("Code Helper (Plugin)", "code"));
        assert!(matches(
            "/Applications/Visual Studio Code.app/Contents/MacOS/Code",
            "code"
        ));
        assert!(matches("claude", "claude"));
        assert!(matches(
            "node\n/usr/lib/node_modules/@anthropic-ai/claude-code/cli.js",
            "claude"
        ));
        assert!(!matches(
            "/opt/@anthropic-ai/claude-code/bin/claude.exe",
            "code"
        ));
        assert!(!matches("codex", "code"));
        assert!(!matches("Xcode", "code"));
        assert!(!matches("kitten", "kitty"));
    }

    fn process(pid: u32, parent: u32, text: &str) -> Process {
        Process {
            pid,
            parent: Some(parent),
            text: String::from(text),
        }
    }

    #[test]
    fn focus_credits_the_app_and_the_best_tool_inside_it() {
        let names = [
            vec![String::from("kitty")],
            vec![String::from("code")],
            vec![String::from("claude")],
            vec![String::from("nvim")],
        ];
        let tools = [
            (2, "Terminal", names[0].as_slice()),
            (1, "VS Code", names[1].as_slice()),
            (0, "Claude Code", names[2].as_slice()),
            (1, "Neovim", names[3].as_slice()),
        ];
        let processes = [
            process(10, 1, "kitty"),
            process(11, 10, "-zsh"),
            process(12, 11, "nvim"),
            process(20, 1, "Code"),
            process(21, 20, "Code Helper"),
            process(30, 1, "Google Chrome"),
            process(31, 30, "claude"),
        ];
        assert_eq!(
            pick_tools(&tools, &processes, Some(10)),
            ["Terminal", "Neovim"]
        );
        assert_eq!(pick_tools(&tools, &processes, Some(20)), ["VS Code"]);
        assert!(pick_tools(&tools, &processes, Some(30)).is_empty());
        assert_eq!(pick_tools(&tools, &processes, None), ["Claude Code"]);
        assert_eq!(pick_tools(&tools, &processes, Some(99)), ["Claude Code"]);
        let mut with_agent = processes.to_vec();
        with_agent.push(process(13, 11, "claude"));
        assert_eq!(
            pick_tools(&tools, &with_agent, Some(10)),
            ["Terminal", "Claude Code"]
        );
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
