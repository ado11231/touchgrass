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
