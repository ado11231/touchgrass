//! Load and check the tool list.

use std::{collections::HashSet, fs, path::Path};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

/// Tools listed in the first config file.
const DEFAULT_TOOLS: &str = "\
[[tools]]
name = \"Claude Code\"
kind = \"ai\"
names = [\"claude\"]

[[tools]]
name = \"Codex\"
kind = \"ai\"
names = [\"codex\"]

[[tools]]
name = \"VS Code\"
kind = \"editor\"
names = [\"code\", \"Code Helper\"]

[[tools]]
name = \"Neovim\"
kind = \"editor\"
names = [\"nvim\"]

[[tools]]
name = \"Terminal\"
kind = \"terminal\"
names = [\"kitty\", \"ghostty\", \"iTerm2\", \"Warp\"]
";

/// Tool kinds in order of priority.
#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// AI tools get the first match.
    Ai,
    /// Editors come before terminals.
    Editor,
    /// Terminals get time when no other tool matches.
    Terminal,
}

/// One tool and the process names that identify it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    /// Name shown in the dashboard.
    pub name: String,
    /// Priority group.
    pub kind: Kind,
    /// Case-insensitive parts of process names or arguments.
    pub names: Vec<String>,
}

/// Settings loaded at startup and once each minute.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Seconds without input before tracking stops.
    pub idle_after: Option<u64>,
    /// Tools in order of preference within each kind.
    pub tools: Vec<Tool>,
}

/// Create the default config only if it is missing.
pub fn initialize(path: &Path, idle_after: u64) -> Result<()> {
    use std::io::Write;
    let parent = path.parent().context("config path has no parent")?;
    fs::create_dir_all(parent).context("could not create config folder")?;
    if path.try_exists().context("could not check config.toml")? {
        return Ok(());
    }
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .context("could not create temporary config file")?;
    write!(temporary, "idle_after = {idle_after}\n\n{DEFAULT_TOOLS}")
        .context("could not write config.toml")?;
    match temporary.persist_noclobber(path) {
        Ok(_) => Ok(()),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error).context("could not create config.toml"),
    }
}

/// Read settings and report the file path on errors.
pub fn load(path: &Path) -> Result<Config> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;
    parse(&contents).with_context(|| format!("invalid {}", path.display()))
}

fn parse(contents: &str) -> Result<Config> {
    let config: Config = toml::from_str(contents).context("could not parse config.toml")?;
    ensure!(config.idle_after != Some(0), "idle_after must be positive");
    let mut names = HashSet::new();
    for tool in &config.tools {
        ensure!(!tool.name.trim().is_empty(), "tool name must not be empty");
        ensure!(
            !tool.name.chars().any(char::is_control),
            "tool name must not contain control characters"
        );
        ensure!(names.insert(&tool.name), "duplicate tool: {}", tool.name);
        ensure!(!tool.names.is_empty(), "{} has no program names", tool.name);
        ensure!(
            tool.names.iter().all(|name| !name.trim().is_empty()),
            "{} has an empty program name",
            tool.name
        );
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() -> Result<()> {
        let config = parse(DEFAULT_TOOLS)?;
        assert_eq!(config.tools.len(), 5);
        assert_eq!(config.tools[0].kind, Kind::Ai);
        Ok(())
    }

    #[test]
    fn rejects_bad_settings() -> Result<()> {
        for contents in [
            "idle_after = 0\ntools = []",
            "idle_after = -1\ntools = []",
            "tools = []\nunknown = 1",
            "[[tools]]\nname = 'A'\nkind = 'other'\nnames = ['a']",
            "[[tools]]\nname = 'A'\nkind = 'ai'\nnames = ['']",
            "[[tools]]\nname = ''\nkind = 'ai'\nnames = ['a']",
        ] {
            assert!(parse(contents).is_err(), "accepted {contents}");
        }
        assert!(
            parse(&format!(
                "{DEFAULT_TOOLS}\n[[tools]]\nname = 'Codex'\nkind = 'ai'\nnames = ['codex']"
            ))
            .is_err()
        );
        Ok(())
    }
    #[test]
    fn initialization_keeps_existing_settings() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("config.toml");
        let idle_after = 45;
        initialize(&path, idle_after)?;
        assert_eq!(load(&path)?.idle_after, Some(idle_after));
        fs::write(&path, "tools = []")?;
        initialize(&path, idle_after)?;
        assert_eq!(fs::read_to_string(path)?, "tools = []");
        Ok(())
    }
}
