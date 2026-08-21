//! Add up saved time and render the terminal dashboard.

use anyhow::{Context, Result};
use chrono::{Datelike, Duration, NaiveDate};
use owo_colors::OwoColorize;
use std::{collections::BTreeMap, fmt::Write};

mod heatmap;

/// Seconds in an hour for labels and heatmap levels.
const HOUR: u64 = 3600;
/// Upper bounds of the three nonzero heatmap levels.
const LEVELS: [u64; 3] = [HOUR, 3 * HOUR, 6 * HOUR];
/// RGB shades from no time to a full day of work.
const COLORS: [(u8, u8, u8); 5] = [
    (43, 43, 43),
    (61, 26, 77),
    (107, 42, 133),
    (165, 63, 204),
    (224, 112, 255),
];
/// Closest shades in the 256-color palette.
const PALETTE: [u8; 5] = [235, 53, 90, 134, 177];
/// Squares used when color is disabled.
const SHADES: [char; 5] = ['·', '░', '▒', '▓', '█'];
/// Length of a tool's share bar.
const BAR_WIDTH: usize = 16;
/// Space before the heatmap's first column.
const GRID_MARGIN: usize = 9;
/// Width of one weekly column, including its gap.
const COLUMN_WIDTH: usize = 2;
/// Title shown at the top of each view.
const TITLE: &str =
    "  ▀█▀ █▀█ █ █ █▀▀ █ █   █▀▀ █▀█ ▄▀█ █▀ █▀\n   █  █▄█ █▄█ █▄▄ █▀█   █▄█ █▀▄ █▀█ ▄█ ▄█";

/// Color support detected by the command entry point.
#[derive(Clone, Copy)]
pub enum Color {
    /// Use shaded text without escape codes.
    None,
    /// Use the standard 256-color palette.
    Palette,
    /// Use exact RGB shades.
    True,
}

fn paint(text: &str, level: usize, color: Color) -> String {
    match color {
        Color::None => String::from(text),
        Color::Palette => format!(
            "{}",
            text.color(owo_colors::XtermColors::from(PALETTE[level]))
        ),
        Color::True => {
            let (red, green, blue) = COLORS[level];
            format!("{}", text.truecolor(red, green, blue))
        }
    }
}

fn level(seconds: u128) -> usize {
    if seconds == 0 {
        return 0;
    }
    LEVELS
        .iter()
        .position(|bound| seconds < u128::from(*bound))
        .map_or(4, |index| index + 1)
}

fn duration(seconds: u128) -> String {
    format!(
        "{}h {:02}m",
        seconds / u128::from(HOUR),
        seconds % u128::from(HOUR) / 60
    )
}

/// Render saved totals using an explicit date and terminal width.
pub fn render(
    days: &BTreeMap<NaiveDate, BTreeMap<String, u64>>,
    installed_on: Option<NaiveDate>,
    today: NaiveDate,
    year: Option<i32>,
    width: usize,
    color: Color,
) -> Result<String> {
    let mut output = format!("\n{}\n\n", paint(TITLE, 4, color));
    heatmap::render(&mut output, days, installed_on, today, year, width, color)?;
    output.push_str("\n         less ");
    for (index, shade) in SHADES.iter().enumerate() {
        let square = match color {
            Color::None => shade.to_string(),
            _ => String::from("■"),
        };
        write!(output, "{} ", paint(&square, index, color))?;
    }
    output.push_str("more\n\n");
    let week_start = today
        .checked_sub_signed(Duration::days(i64::from(
            today.weekday().num_days_from_monday(),
        )))
        .context("week is outside the supported date range")?;
    let total = |start: NaiveDate, end: NaiveDate| -> u128 {
        days.range(start..=end)
            .flat_map(|(_, tools)| tools.values())
            .map(|seconds| u128::from(*seconds))
            .sum()
    };
    let (start, end, heading) = if let Some(year) = year {
        let start = NaiveDate::from_ymd_opt(year, 1, 1).context("invalid year")?;
        let end = NaiveDate::from_ymd_opt(year, 12, 31).context("invalid year")?;
        writeln!(output, "  {year} total  {}", duration(total(start, end)))?;
        (start, end, format!("{year} by tool"))
    } else {
        writeln!(output, "  Today      {:>12}", duration(total(today, today)))?;
        writeln!(
            output,
            "  This week  {:>12}",
            duration(total(week_start, today))
        )?;
        writeln!(
            output,
            "  All time   {:>12}",
            duration(total(NaiveDate::MIN, today))
        )?;
        (today, today, String::from("Today by tool"))
    };
    writeln!(output, "\n  {heading}")?;
    let mut totals: BTreeMap<&str, u128> = BTreeMap::new();
    for tools in days.range(start..=end).map(|(_, tools)| tools) {
        for (name, seconds) in tools {
            *totals.entry(name).or_default() += u128::from(*seconds);
        }
    }
    let mut tools: Vec<_> = totals
        .into_iter()
        .filter(|(_, seconds)| *seconds > 0)
        .collect();
    tools.sort_by(|(left_name, left_time), (right_name, right_time)| {
        right_time
            .cmp(left_time)
            .then_with(|| left_name.cmp(right_name))
    });
    let name_width = tools
        .iter()
        .map(|(name, _)| name.chars().count())
        .max()
        .unwrap_or(0);
    let sum: u128 = tools.iter().map(|(_, seconds)| seconds).sum();
    if tools.is_empty() {
        output.push_str("  No time recorded.\n");
    }
    for (name, seconds) in tools {
        let filled = ((seconds * BAR_WIDTH as u128 + sum / 2) / sum) as usize;
        let bar = format!(
            "{}{}",
            paint(&"█".repeat(filled), 4, color),
            paint(&"░".repeat(BAR_WIDTH - filled), 0, color)
        );
