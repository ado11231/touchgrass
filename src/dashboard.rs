//! Add up saved time and render the terminal dashboard.

use anyhow::{Context, Result};
use chrono::{Datelike, Duration, NaiveDate};
use owo_colors::OwoColorize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

mod heatmap;

/// Seconds in an hour for labels and heatmap levels.
const HOUR: u64 = 3600;
/// Upper bounds of the three nonzero heatmap levels.
const LEVELS: [u64; 3] = [HOUR, 3 * HOUR, 6 * HOUR];
/// RGB shades from no time to a full day of work.
const COLORS: [(u8, u8, u8); 5] = [
    (43, 43, 43),
    (82, 36, 104),
    (107, 42, 133),
    (165, 63, 204),
    (224, 112, 255),
];
/// Closest shades in the 256-color palette.
const PALETTE: [u8; 5] = [235, 54, 90, 134, 177];
/// Labels, the heatmap border, and other plain text.
const WHITE: (u8, u8, u8) = (255, 255, 255);
/// Bright white in the 256-color palette.
const WHITE_PALETTE: u8 = 15;
/// Squares used when color is disabled.
const SHADES: [char; 5] = ['·', '░', '▒', '▓', '█'];
/// One heatmap day across two cells, drawn with 2x3 sextant blocks.
///
/// It fills three of four half-columns and the lower two of three rows, so
/// each square is 1.5 cells wide with a half-cell gap, and sits level with
/// lowercase text.
const CELL: &str = "\u{1FB39}\u{1FB13}";
/// Length of a tool's share bar.
const BAR_WIDTH: usize = 24;
/// Indent, border, day labels, and padding around the heatmap squares.
const GRID_MARGIN: usize = 9;
/// Width of one weekly column, including its gap.
const COLUMN_WIDTH: usize = 2;
/// Width of one summary card.
const CARD_WIDTH: usize = 18;
/// Indent shared by every section.
const INDENT: &str = "  ";
/// Compact title for terminals too narrow for the large one.
const SMALL_TITLE: &str =
    "  ▀█▀ █▀█ █ █ █▀▀ █ █   █▀▀ █▀█ ▄▀█ █▀ █▀\n   █  █▄█ █▄█ █▄▄ █▀█   █▄█ █▀▄ █▀█ ▄█ ▄█";
/// Large title letters in the ANSI Shadow style.
const GLYPHS: [(char, [&str; 6]); 9] = [
    (
        'T',
        [
            "████████╗",
            "╚══██╔══╝",
            "   ██║   ",
            "   ██║   ",
            "   ██║   ",
            "   ╚═╝   ",
        ],
    ),
    (
        'O',
        [
            " ██████╗ ",
            "██╔═══██╗",
            "██║   ██║",
            "██║   ██║",
            "╚██████╔╝",
            " ╚═════╝ ",
        ],
    ),
    (
        'U',
        [
            "██╗   ██╗",
            "██║   ██║",
            "██║   ██║",
            "██║   ██║",
            "╚██████╔╝",
            " ╚═════╝ ",
        ],
    ),
    (
        'C',
        [
            " ██████╗",
            "██╔════╝",
            "██║     ",
            "██║     ",
            "╚██████╗",
            " ╚═════╝",
        ],
    ),
    (
        'H',
        [
            "██╗  ██╗",
            "██║  ██║",
            "███████║",
            "██╔══██║",
            "██║  ██║",
            "╚═╝  ╚═╝",
        ],
    ),
    (
        'G',
        [
            " ██████╗ ",
            "██╔════╝ ",
            "██║  ███╗",
            "██║   ██║",
            "╚██████╔╝",
            " ╚═════╝ ",
        ],
    ),
    (
        'R',
        [
            "██████╗ ",
            "██╔══██╗",
            "██████╔╝",
            "██╔══██╗",
            "██║  ██║",
            "╚═╝  ╚═╝",
        ],
    ),
    (
        'A',
        [
            " █████╗ ",
            "██╔══██╗",
            "███████║",
            "██╔══██║",
            "██║  ██║",
            "╚═╝  ╚═╝",
        ],
    ),
    (
        'S',
        [
            "███████╗",
            "██╔════╝",
            "███████╗",
            "╚════██║",
            "███████║",
            "╚══════╝",
        ],
    ),
];
/// Word spelled by the large title.
const WORD: &str = "TOUCHGRASS";
/// Most spaces added between large title letters on wide terminals.
const MAX_LETTER_GAP: usize = 2;

/// Color support detected by the command entry point.
#[derive(Clone, Copy, PartialEq, Eq)]
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

fn white(text: &str, color: Color) -> String {
    match color {
        Color::None => String::from(text),
        Color::Palette => format!(
            "{}",
            text.color(owo_colors::XtermColors::from(WHITE_PALETTE))
        ),
        Color::True => format!("{}", text.truecolor(WHITE.0, WHITE.1, WHITE.2)),
    }
}

fn strong(text: &str, color: Color) -> String {
    match color {
        Color::None => String::from(text),
        _ => format!("{}", paint(text, 4, color).bold()),
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

/// Large letters with solid faces in bright magenta and a darker shadow.
fn title(width: usize, color: Color) -> String {
    let letters: Vec<_> = WORD
        .chars()
        .filter_map(|letter| GLYPHS.iter().find(|(glyph, _)| *glyph == letter))
        .map(|(_, rows)| rows)
        .collect();
    let letters_width: usize = letters.iter().map(|rows| rows[0].chars().count()).sum();
    let gaps = letters.len().saturating_sub(1);
    let room = width.saturating_sub(INDENT.len());
    if room < letters_width {
        return paint(SMALL_TITLE, 4, color);
    }
    let gap = ((room - letters_width) / gaps.max(1)).min(MAX_LETTER_GAP);
    let mut lines = Vec::new();
    for row in 0..6 {
        let text = letters
            .iter()
            .map(|rows| rows[row])
            .collect::<Vec<_>>()
            .join(&" ".repeat(gap));
        let mut line = String::from(INDENT);
        let mut run = String::new();
        let mut run_level = None;
        for character in text.chars() {
            let next = match character {
                ' ' => None,
                '█' => Some(4),
                _ => Some(2),
            };
            if next != run_level && !run.is_empty() {
                line.push_str(&run_level.map_or(run.clone(), |level| paint(&run, level, color)));
                run.clear();
            }
            run_level = next;
            run.push(character);
        }
        line.push_str(&run_level.map_or(run.clone(), |level| paint(&run, level, color)));
        lines.push(line.trim_end().to_string());
    }
    lines.join("\n")
}

/// Saved seconds with lookups that agree on what a day's total is.
struct Totals<'a> {
    days: &'a BTreeMap<NaiveDate, BTreeMap<String, u64>>,
    active: &'a BTreeMap<NaiveDate, u64>,
}

impl Totals<'_> {
    /// Wall-clock time on one date, or the sum of tools for older data.
    fn day(&self, date: NaiveDate) -> u128 {
        self.active.get(&date).map_or_else(
            || {
                self.days
                    .get(&date)
                    .map(|tools| tools.values().map(|seconds| u128::from(*seconds)).sum())
                    .unwrap_or(0)
            },
            |seconds| u128::from(*seconds),
        )
    }

    fn dates(&self, start: NaiveDate, end: NaiveDate) -> BTreeSet<NaiveDate> {
        self.days
            .range(start..=end)
            .map(|(date, _)| *date)
            .chain(self.active.range(start..=end).map(|(date, _)| *date))
            .collect()
    }

    fn range(&self, start: NaiveDate, end: NaiveDate) -> u128 {
        self.dates(start, end)
            .into_iter()
            .map(|date| self.day(date))
            .sum()
    }

    fn tools(&self, start: NaiveDate, end: NaiveDate) -> Vec<(&str, u128)> {
        let mut totals: BTreeMap<&str, u128> = BTreeMap::new();
        for tools in self.days.range(start..=end).map(|(_, tools)| tools) {
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
        tools
    }
}

/// Write labelled values side by side, like small cards.
fn cards(output: &mut String, cards: &[(String, String)], color: Color) -> Result<()> {
    let mut labels = String::from(INDENT);
    let mut values = String::from(INDENT);
    for (index, (label, value)) in cards.iter().enumerate() {
        let last = index + 1 == cards.len();
        let pad = |text: &str| {
            if last {
                String::new()
            } else {
                " ".repeat(CARD_WIDTH.saturating_sub(text.chars().count()))
            }
        };
        labels.push_str(&white(label, color));
        labels.push_str(&pad(label));
        values.push_str(&strong(value, color));
        values.push_str(&pad(value));
    }
    writeln!(output, "{labels}\n{values}")?;
    Ok(())
}

/// Render saved totals using an explicit date and terminal width.
pub fn render(
    days: &BTreeMap<NaiveDate, BTreeMap<String, u64>>,
    active: &BTreeMap<NaiveDate, u64>,
    today: NaiveDate,
    year: Option<i32>,
    width: usize,
    color: Color,
) -> Result<String> {
    let totals = Totals { days, active };
    let mut output = format!("\n{}\n\n", title(width, color));
    let grid_width = heatmap::render(&mut output, &totals, today, year, width, color)?;

    let mut squares = String::new();
    for (index, shade) in SHADES.iter().enumerate() {
        let square = match color {
            Color::None => format!("{shade} "),
            _ => String::from(CELL),
        };
        squares.push_str(&paint(&square, index, color));
    }
    // Plain squares already end with a space; sextant squares end with half a cell.
    let more = if color == Color::None {
        "more"
    } else {
        " more"
    };
    let legend_width = "less ".len() + SHADES.len() * COLUMN_WIDTH + more.len();
    // End the legend under the heatmap's right border.
    let legend_pad = grid_width.saturating_sub(legend_width);
    writeln!(
        output,
        "{}{} {squares}{}\n",
        " ".repeat(legend_pad),
        white("less", color),
        white(more, color)
    )?;

    let (start, end) = if let Some(year) = year {
        let start = NaiveDate::from_ymd_opt(year, 1, 1).context("invalid year")?;
        let end = NaiveDate::from_ymd_opt(year, 12, 31).context("invalid year")?;
        let dates = totals.dates(start, end);
        let active_days = dates.iter().filter(|date| totals.day(**date) > 0).count();
        let best = dates
            .iter()
            .map(|date| (totals.day(*date), *date))
            .filter(|(seconds, _)| *seconds > 0)
            .max_by(|(left, left_date), (right, right_date)| {
                left.cmp(right).then_with(|| right_date.cmp(left_date))
            });
        cards(
            &mut output,
            &[
                (format!("{year} TOTAL"), duration(totals.range(start, end))),
                (String::from("ACTIVE DAYS"), active_days.to_string()),
                (
                    String::from("BEST DAY"),
                    best.map_or_else(
                        || String::from("—"),
                        |(seconds, date)| {
                            format!("{}  {}", date.format("%b %-d"), duration(seconds))
                        },
                    ),
                ),
            ],
            color,
        )?;
        (start, end)
    } else {
        let week_start = today
            .checked_sub_signed(Duration::days(i64::from(
                today.weekday().num_days_from_monday(),
            )))
            .context("week is outside the supported date range")?;
        cards(
            &mut output,
            &[
                (String::from("TODAY"), duration(totals.range(today, today))),
                (
                    String::from("THIS WEEK"),
                    duration(totals.range(week_start, today)),
                ),
                (
                    String::from("ALL TIME"),
                    duration(totals.range(NaiveDate::MIN, today)),
                ),
            ],
            color,
        )?;
        (today, today)
    };
    output.push('\n');
    let tools = totals.tools(start, end);
    if tools.is_empty() {
        writeln!(output, "{INDENT}No time recorded.")?;
    }
    let name_width = tools
        .iter()
        .map(|(name, _)| name.chars().count())
        .max()
        .unwrap_or(0);
    // Tools overlap, so each bar is a share of active time, not of the tools.
    let whole = totals
        .range(start, end)
        .max(tools.first().map_or(0, |(_, seconds)| *seconds))
        .max(1);
    for (name, seconds) in tools {
        let filled =
            ((seconds * BAR_WIDTH as u128 + whole / 2) / whole).min(BAR_WIDTH as u128) as usize;
        let percent = (seconds * 100 + whole / 2) / whole;
        writeln!(
            output,
            "{INDENT}{name:name_width$}  {}{}  {:>9}  {}",
            paint(&"█".repeat(filled), 4, color),
            paint(&"░".repeat(BAR_WIDTH - filled), 0, color),
            duration(seconds),
            white(&format!("{percent:>3}%"), color),
        )?;
    }
    // Leave room before the shell prompt.
    output.push('\n');
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> Result<NaiveDate> {
        Ok(NaiveDate::parse_from_str(value, "%Y-%m-%d")?)
    }

    fn values_line(text: &str, label: &str) -> Option<String> {
        let mut lines = text.lines();
        lines.find(|line| line.trim_start().starts_with(label))?;
        lines.next().map(String::from)
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
    fn large_title_rows_line_up_and_fall_back_when_narrow() {
        for (letter, rows) in GLYPHS {
            let width = rows[0].chars().count();
            assert!(
                rows.iter().all(|row| row.chars().count() == width),
                "ragged {letter}"
            );
        }
        let wide = title(200, Color::None);
        assert_eq!(wide.lines().count(), 6);
        assert!(wide.lines().all(|line| line.chars().count() <= 200));
        assert!(wide.contains('╗'));
        assert!(title(60, Color::None).starts_with("  ▀█▀"));
        assert!(title(200, Color::True).contains("38;2;107;42;133"));
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
        let text = render(&days, &BTreeMap::new(), today, None, 120, Color::None)?;
        let values = values_line(&text, "TODAY").context("missing summary")?;
        let values: Vec<_> = values
            .split("  ")
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .collect();
        assert_eq!(values, ["1h 00m", "2h 00m", "3h 00m"]);
        assert!(!text.contains("Old tool"));
        assert!(!text.contains('\x1b'));
        Ok(())
    }

    #[test]
    fn overlapping_tools_use_active_time_for_totals_and_shares() -> Result<()> {
        let today = date("2026-11-04")?;
        let days = BTreeMap::from([(
            today,
            BTreeMap::from([
                (String::from("Terminal"), 2 * HOUR),
                (String::from("Claude Code"), HOUR),
            ]),
        )]);
        let active = BTreeMap::from([(today, 2 * HOUR)]);
        let text = render(&days, &active, today, None, 120, Color::None)?;
        let values = values_line(&text, "TODAY").context("missing summary")?;
        assert!(values.trim_start().starts_with("2h 00m"));
        let terminal = text
            .lines()
            .find(|line| line.contains("Terminal"))
            .context("missing Terminal")?;
        assert!(terminal.ends_with("100%"));
        let claude = text
            .lines()
            .find(|line| line.contains("Claude Code"))
            .context("missing Claude Code")?;
        assert!(claude.ends_with(" 50%"));
        Ok(())
    }

    #[test]
    fn default_view_shows_a_full_window_from_the_start() -> Result<()> {
        let today = date("2026-11-04")?;
        let days = BTreeMap::from([(today, BTreeMap::from([(String::from("Codex"), HOUR)]))]);
        let text = render(&days, &BTreeMap::new(), today, None, 200, Color::None)?;
        // 51 full weeks plus Monday through Wednesday, and one legend square.
        assert_eq!(text.matches('·').count(), 51 * 7 + 2 + 1);
        Ok(())
    }

    #[test]
    fn year_contains_leap_day_and_all_months() -> Result<()> {
        let today = date("2025-01-01")?;
        let text = render(
            &BTreeMap::new(),
            &BTreeMap::new(),
            today,
            Some(2024),
            80,
            Color::None,
        )?;
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
            let text = render(
                &BTreeMap::new(),
                &BTreeMap::new(),
                today,
                None,
                width,
                Color::None,
            )?;
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
        let text = render(&days, &BTreeMap::new(), today, Some(2026), 120, Color::None)?;
        let values = values_line(&text, "2026 TOTAL").context("missing summary")?;
        assert!(values.trim_start().starts_with("4h 00m"));
        assert!(values.contains("Dec 31  4h 00m"));
        assert!(!text.contains("Old"));
        assert!(
            text.find("Large").context("missing Large")?
                < text.find("Small").context("missing Small")?
        );
        Ok(())
    }
}
