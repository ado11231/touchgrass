//! Track active time in dev tools and show daily totals.

use anyhow::{Context, Result, bail};
use chrono::Local;
use std::{
    env, fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
};

mod autostart;
mod config;
mod dashboard;
mod focus;
mod idle;
mod storage;
mod tracker;

/// Use a readable width when stdout has no terminal size.
const DEFAULT_WIDTH: usize = 120;
/// Keep year arithmetic within the supported date range.
const FIRST_YEAR: i32 = 1;
/// Last four-digit calendar year accepted by the CLI.
const LAST_YEAR: i32 = 9999;
/// Commands supported by this version.
const HELP: &str = "touchgrass - time in your dev tools

Usage:
  touchgrass             Show recent time
  touchgrass YEAR        Show a calendar year
  touchgrass track       Track time in this terminal
  touchgrass install     Start tracking now and at login
  touchgrass uninstall   Stop the login service
  touchgrass help        Show this help
";

fn main() {
    if let Err(error) = run() {
        if error
            .downcast_ref::<io::Error>()
            .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe)
        {
            return;
        }
        eprintln!("touchgrass: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut arguments = env::args_os().skip(1);
    let command = arguments
        .next()
        .map(|value| {
            value
                .into_string()
                .map_err(|_| anyhow::anyhow!("command is not valid UTF-8"))
        })
        .transpose()?;
    if arguments.next().is_some() {
        bail!("too many arguments; run touchgrass help");
    }
    match command.as_deref() {
        Some("help" | "--help" | "-h") => {
            print_output(HELP)?;
            return Ok(());
        }
        Some("uninstall") => {
            autostart::uninstall()?;
            return Ok(());
        }
        Some("track" | "install") | None => (),
        Some(value) => {
            parse_year(value)?;
        }
    }
    let config_path = dirs::config_dir()
        .context("could not find config folder")?
        .join("touchgrass/config.toml");
    let data_folder = dirs::data_local_dir()
        .context("could not find data folder")?
        .join("touchgrass");
    config::initialize(&config_path, tracker::DEFAULT_IDLE_AFTER)?;
    match command.as_deref() {
        Some("track") => track(config_path, data_folder),
        Some("install") => {
            config::load(&config_path)?;
            autostart::install(&env::current_exe().context("could not find executable")?)
        }
        other => {
            // Bad settings should not hide time that was already saved.
            if let Err(error) = config::load(&config_path) {
                eprintln!("touchgrass: {error:#}");
            }
            let year = other.map(parse_year).transpose()?;
            let (data, missing) = match storage::load(&data_folder.join("data.json"))? {
                storage::Loaded::Missing => (storage::Data::default(), true),
                storage::Loaded::Present(data) => (data, false),
                storage::Loaded::Invalid(error) => return Err(error).context(
                    "saved time is invalid; run touchgrass track to preserve it and start fresh",
                ),
            };
            let width = terminal_size::terminal_size()
                .map(|(terminal_size::Width(width), _)| usize::from(width))
                .unwrap_or(DEFAULT_WIDTH);
            let output = dashboard::render(
                &data.days,
                &data.active,
                Local::now().date_naive(),
                year,
                width,
                color(),
            )?;
            print_output(&output)?;
            if missing {
                print_output("  No saved time yet. Run touchgrass install to start tracking.\n\n")?;
            }
            Ok(())
        }
    }
}

fn parse_year(value: &str) -> Result<i32> {
    let year: i32 = value
        .parse()
        .context("unknown command; run touchgrass help")?;
    if !(FIRST_YEAR..=LAST_YEAR).contains(&year) {
        bail!("year must be between {FIRST_YEAR} and {LAST_YEAR}");
    }
    Ok(year)
}

fn color() -> dashboard::Color {
    if !io::stdout().is_terminal()
        || env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
        || env::var("TERM").is_ok_and(|value| value == "dumb")
    {
        dashboard::Color::None
    } else if env::var("COLORTERM").is_ok_and(|value| value == "truecolor" || value == "24bit") {
        dashboard::Color::True
    } else {
        dashboard::Color::Palette
    }
}

fn print_output(output: &str) -> Result<()> {
    io::stdout()
        .lock()
        .write_all(output.as_bytes())
        .context("could not print output")
}

fn track(config_path: PathBuf, data_folder: PathBuf) -> Result<()> {
    let mut config = config::load(&config_path)?;
    fs::create_dir_all(&data_folder).context("could not create data folder")?;
    let _lock = tracker::acquire_lock(&data_folder.join("touchgrass.lock"))?;
    let shutdown = tracker::shutdown_signal()?;
    let data_path = data_folder.join("data.json");
    let mut data = match storage::load(&data_path)? {
        storage::Loaded::Missing => storage::Data::default(),
        storage::Loaded::Present(data) => data,
        storage::Loaded::Invalid(error) => {
            let backup = storage::backup(&data_path)?;
            eprintln!("touchgrass: {error:#}; preserved at {}", backup.display());
            storage::Data::default()
        }
    };
    data.installed_on.get_or_insert(Local::now().date_naive());
    storage::save(&data_path, &data)?;
    let mut system = sysinfo::System::new().context("failed to initialize system info")?;
    let mut warned = false;
    tracker::run(shutdown, |event| {
        match event {
            tracker::Event::Sample => {
                let limit = config.idle_after.unwrap_or(tracker::DEFAULT_IDLE_AFTER);
                if idle::active(limit, &mut warned)? {
                    let processes = tracker::processes(&mut system);
                    let tools: Vec<_> = config
                        .tools
                        .iter()
                        .map(|tool| {
                            let priority = match tool.kind {
                                config::Kind::Ai => 0,
                                config::Kind::Editor => 1,
                                config::Kind::Terminal => 2,
                            };
                            (priority, tool.name.as_str(), tool.names.as_slice())
                        })
                        .collect();
                    let picked = tracker::pick_tools(&tools, &processes, focus::pid());
                    let today = Local::now().date_naive();
                    for tool in &picked {
                        tracker::record(&mut data.days, today, tool)?;
                    }
                    if !picked.is_empty() {
                        tracker::record_active(&mut data.active, today)?;
                    }
                }
            }
            tracker::Event::Maintain => {
                storage::save(&data_path, &data)?;
                match config::load(&config_path) {
                    Ok(updated) => config = updated,
                    Err(error) => eprintln!("touchgrass: keeping previous config: {error:#}"),
                }
            }
            tracker::Event::Stop => storage::save(&data_path, &data)?,
        }
        Ok(())
    })
}
