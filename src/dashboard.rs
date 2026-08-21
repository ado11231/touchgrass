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
        writeln!(
            output,
            "  {name:name_width$}  {bar}  {:>10}",
            duration(seconds)
        )?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> Result<NaiveDate> {
        Ok(NaiveDate::parse_from_str(value, "%Y-%m-%d")?)
    }

    #[test]
    fn color_levels_include_exact_boundaries() -> Result<()> {
        assert_eq!(level(0), 0);
        assert_eq!(level(u128::from(HOUR - 1)), 1);
        for (index, bound) in LEVELS.iter().enumerate() {
            assert_eq!(level(u128::from(*bound)), index + 2);
        }
        assert_eq!(paint("x", 4, Color::None), "x");
        assert!(paint("x", 4, Color::True).contains("38;2;224;112;255"));
        assert!(paint("x", 4, Color::Palette).contains("38;5;177"));
        Ok(())
    }

    #[test]
    fn week_crosses_new_year_without_resetting_totals() -> Result<()> {
        let previous = date("2026-12-31")?;
        let today = date("2027-01-01")?;
        let old = date("2025-01-01")?;
        let days = BTreeMap::from([
            (old, BTreeMap::from([(String::from("Old tool"), HOUR)])),
            (previous, BTreeMap::from([(String::from("Codex"), HOUR)])),
            (today, BTreeMap::from([(String::from("Codex"), HOUR)])),
        ]);
        let text = render(&days, Some(old), today, None, 120, Color::None)?;
        let lines: Vec<_> = text.lines().map(str::trim).collect();
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("Today ") && line.ends_with("1h 00m"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("This week") && line.ends_with("2h 00m"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("All time") && line.ends_with("3h 00m"))
        );
        assert!(!text.contains("Old tool"));
        assert!(!text.contains('\x1b'));
        Ok(())
    }

    #[test]
    fn year_contains_leap_day_and_all_months() -> Result<()> {
        let today = date("2025-01-01")?;
        let text = render(&BTreeMap::new(), None, today, Some(2024), 80, Color::None)?;
        assert_eq!(text.matches('·').count(), 367);
        for month in [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ] {
            assert!(text.contains(month), "missing {month}");
        }
        assert!(!text.contains('\x1b'));
        Ok(())
    }

    #[test]
    fn no_data_and_tiny_terminals_render_safely() -> Result<()> {
        let today = date("2026-08-14")?;
        for width in [0, 10, 40, 80, 120] {
            let text = render(&BTreeMap::new(), None, today, None, width, Color::None)?;
            assert!(text.contains("No time recorded."));
            assert!(text.contains("0h 00m"));
        }
        Ok(())
    }

    #[test]
    fn year_tools_are_sorted_and_totals_ignore_other_years() -> Result<()> {
        let today = date("2026-12-31")?;
        let days = BTreeMap::from([
            (
                today,
                BTreeMap::from([
                    (String::from("Small"), HOUR),
                    (String::from("Large"), HOUR * 3),
                ]),
            ),
            (
                date("2025-01-01")?,
                BTreeMap::from([(String::from("Old"), HOUR)]),
            ),
        ]);
        let text = render(&days, None, today, Some(2026), 120, Color::None)?;
        assert!(text.contains("2026 total  4h 00m"));
        assert!(!text.contains("Old"));
        assert!(
            text.find("Large").context("missing Large")?
                < text.find("Small").context("missing Small")?
        );
        Ok(())
    }
}
