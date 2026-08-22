//! Load and save daily time without printing.

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Saved time and the date tracking began.
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Data {
    /// First day of tracking, even if no tools matched.
    #[serde(default)]
    pub installed_on: Option<NaiveDate>,
    /// Seconds per tool on each local date.
    #[serde(default)]
    pub days: BTreeMap<NaiveDate, BTreeMap<String, u64>>,
    /// Wall-clock seconds in any tool on each local date.
    ///
    /// Tools can overlap, such as Claude Code inside kitty, so totals come
    /// from here. Days saved before this field existed use the sum of tools.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub active: BTreeMap<NaiveDate, u64>,
}

/// A read keeps missing data separate from invalid data.
pub enum Loaded {
    /// No saved file exists yet.
    Missing,
    /// The file parsed successfully.
    Present(Data),
    /// The file exists but cannot be parsed.
    Invalid(anyhow::Error),
}

/// Read saved time without changing any files.
pub fn load(path: &Path) -> Result<Loaded> {
    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Loaded::Missing),
        Err(error) => {
            return Err(error).with_context(|| format!("could not read {}", path.display()));
        }
    };
    let data = serde_json::from_slice::<Data>(&contents)
        .context("could not parse data.json")
        .and_then(|mut data| {
            for totals in data.days.values() {
                for name in totals.keys() {
                    ensure!(!name.trim().is_empty(), "saved tool name is empty");
                    ensure!(
                        !name.chars().any(char::is_control),
                        "saved tool name has control characters"
                    );
                }
            }
            if data.installed_on.is_none() {
                data.installed_on = data.days.keys().next().copied();
            }
            Ok(data)
        });
    Ok(match data {
        Ok(data) => Loaded::Present(data),
        Err(error) => Loaded::Invalid(error),
    })
}

/// Replace saved data only after the new file has been flushed.
pub fn save(path: &Path, data: &Data) -> Result<()> {
    let parent = path.parent().context("data path has no parent")?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).context("could not create temporary data file")?;
    serde_json::to_writer_pretty(&mut temporary, data).context("could not write saved time")?;
    temporary
        .write_all(b"\n")
        .context("could not finish saved time")?;
    temporary
        .as_file()
        .sync_all()
        .context("could not flush saved time")?;
    temporary
        .persist(path)
        .context("could not replace data.json")?;
    fs::File::open(parent)?
        .sync_all()
        .context("could not flush data folder")?;
    Ok(())
}

/// Preserve corrupt data before the tracker starts a new file.
pub fn backup(path: &Path) -> Result<PathBuf> {
    for number in 0..u32::MAX {
        let name = if number == 0 {
            String::from("data.json.bak")
        } else {
            format!("data.json.bak.{number}")
        };
        let destination = path.with_file_name(name);
        // A hard link avoids replacing an older backup during recovery.
        match fs::hard_link(path, &destination) {
            Ok(()) => {
                fs::remove_file(path).context("could not finish moving corrupt data")?;
                return Ok(destination);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error).context("could not preserve corrupt data.json"),
        }
    }
    bail!("no free backup name for data.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_missing_empty_and_invalid_data() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("data.json");
        assert!(matches!(load(&path)?, Loaded::Missing));
        fs::write(&path, b"")?;
        assert!(matches!(load(&path)?, Loaded::Invalid(_)));
        assert_eq!(fs::read(&path)?, b"");
        fs::write(&path, b"{}")?;
        assert!(matches!(
            load(&path)?,
            Loaded::Present(Data {
                installed_on: None,
                ..
            })
        ));
        fs::write(&path, b"{bad json")?;
        assert!(matches!(load(&path)?, Loaded::Invalid(_)));
        assert_eq!(fs::read(&path)?, b"{bad json");
        Ok(())
    }

    #[test]
    fn round_trip_keeps_time_and_install_date() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("data.json");
        let date = NaiveDate::parse_from_str("2026-08-14", "%Y-%m-%d")?;
        let data = Data {
            installed_on: Some(date),
            days: BTreeMap::from([(date, BTreeMap::from([(String::from("Claude Code"), 75)]))]),
            active: BTreeMap::from([(date, 75)]),
        };
        save(&path, &data)?;
        match load(&path)? {
            Loaded::Present(actual) => assert_eq!(actual, data),
            _ => bail!("saved file did not load"),
        }
        assert_eq!(fs::read_dir(directory.path())?.count(), 1);
        Ok(())
    }

    #[test]
    fn recovers_without_replacing_old_backups() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("data.json");
        fs::write(&path, b"first broken file")?;
        let first = backup(&path)?;
        fs::write(&path, b"second broken file")?;
        let second = backup(&path)?;
        assert_ne!(first, second);
        assert_eq!(fs::read(first)?, b"first broken file");
        assert_eq!(fs::read(second)?, b"second broken file");
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn uses_first_day_when_install_date_is_missing() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("data.json");
        fs::write(&path, br#"{"days":{"2026-08-14":{"Codex":10}}}"#)?;
        let Loaded::Present(data) = load(&path)? else {
            bail!("valid data did not load");
        };
        assert_eq!(data.installed_on, data.days.keys().next().copied());
        Ok(())
    }
}
