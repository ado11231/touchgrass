//! Arrange dates into Monday-based heatmap columns.

use super::*;

/// Number of week columns in the sliding view.
const WINDOW_WEEKS: i64 = 52;
/// Width of the Mon, Wed, and Fri labels.
const DAY_LABEL_WIDTH: usize = 3;
/// Days in a complete heatmap column.
const WEEK_DAYS: i64 = 7;

fn monday(date: NaiveDate) -> Result<NaiveDate> {
    date.checked_sub_signed(Duration::days(i64::from(
        date.weekday().num_days_from_monday(),
    )))
    .context("heatmap date is outside the supported range")
}

/// Write the date grid and return its width in columns.
///
/// The default view always shows a full window ending this week, like GitHub,
/// shortened only when the terminal is too narrow. A year view splits into
/// panels instead so no day is hidden.
pub(super) fn render(
    output: &mut String,
    totals: &Totals,
    today: NaiveDate,
    year: Option<i32>,
    width: usize,
    color: Color,
) -> Result<usize> {
    let columns = (width.saturating_sub(GRID_MARGIN) / COLUMN_WIDTH).max(1);
    let ranges = if let Some(year) = year {
        let start = NaiveDate::from_ymd_opt(year, 1, 1).context("invalid year")?;
        let end = NaiveDate::from_ymd_opt(year, 12, 31).context("invalid year")?;
        let weeks = (end - monday(start)?).num_days() / WEEK_DAYS + 1;
        if weeks as usize > columns {
            vec![
                (
                    start,
                    NaiveDate::from_ymd_opt(year, 6, 30).context("invalid year")?,
                ),
                (
                    NaiveDate::from_ymd_opt(year, 7, 1).context("invalid year")?,
                    end,
                ),
            ]
        } else {
            vec![(start, end)]
        }
    } else {
        let weeks = WINDOW_WEEKS.min(columns as i64);
        let start = monday(today)?
            .checked_sub_signed(Duration::weeks(weeks - 1))
            .context("invalid heatmap range")?;
        vec![(start, today)]
    };
    let mut widest = 0;
    for (start, end) in ranges {
        let mut first = monday(start)?;
        while first <= end {
            let remaining = (end - first).num_days() / WEEK_DAYS + 1;
            let count = columns.min(remaining as usize);
            widest = widest.max(count);
            panel(output, totals, (start, end, today, first), count, color)?;
            first = first
                .checked_add_signed(Duration::weeks(count as i64))
                .context("invalid heatmap range")?;
            if first <= end {
                output.push('\n');
            }
        }
    }
    Ok(GRID_MARGIN + widest * COLUMN_WIDTH)
}

fn panel(
    output: &mut String,
    totals: &Totals,
    dates: (NaiveDate, NaiveDate, NaiveDate, NaiveDate),
    columns: usize,
    color: Color,
) -> Result<()> {
    let (start, end, today, first) = dates;
    let grid = columns * COLUMN_WIDTH;
    // One space between the border, the day labels, the squares, and the border.
    let inner = 1 + DAY_LABEL_WIDTH + 1 + grid;
    let mut labels = vec![' '; grid];
    let mut next_label = 0;
    for column in 0..columns {
        let date = first + Duration::weeks(column as i64);
        let last_day = (date + Duration::days(WEEK_DAYS - 1)).min(end);
        if column == 0 || date.month() != last_day.month() || date.day() == 1 {
            let label = last_day.format("%b").to_string();
            // Keep a late label inside the border instead of past it.
            let position = (column * COLUMN_WIDTH).min(grid.saturating_sub(label.len()));
            if position >= next_label && position + label.len() <= grid {
                for (offset, character) in label.chars().enumerate() {
                    labels[position + offset] = character;
                }
                next_label = position + label.len() + 1;
            }
        }
    }
    let border = |text: &str| white(text, color);
    writeln!(
        output,
        "{INDENT}{}",
        border(&format!("╭{}╮", "─".repeat(inner)))
    )?;
    writeln!(
        output,
        "{INDENT}{} {} {}{}",
        border("│"),
        " ".repeat(DAY_LABEL_WIDTH),
        white(&labels.into_iter().collect::<String>(), color),
        border("│")
    )?;
    for weekday in 0..WEEK_DAYS {
        let label = match weekday {
            0 => "Mon",
            2 => "Wed",
            4 => "Fri",
            _ => "   ",
        };
        write!(output, "{INDENT}{} {} ", border("│"), white(label, color))?;
        for column in 0..columns {
            let date = first + Duration::days(column as i64 * WEEK_DAYS + weekday);
            if date < start || date > end {
                output.push_str("  ");
                continue;
            }
            let seconds = if date > today { 0 } else { totals.day(date) };
            let level = level(seconds);
            let square = match color {
                Color::None => format!("{} ", SHADES[level]),
                _ => String::from(CELL),
            };
            output.push_str(&paint(&square, level, color));
        }
        writeln!(output, "{}", border("│"))?;
    }
    // A blank row balances the month labels above the squares.
    writeln!(
        output,
        "{INDENT}{}{}{}",
        border("│"),
        " ".repeat(inner),
        border("│")
    )?;
    writeln!(
        output,
        "{INDENT}{}",
        border(&format!("╰{}╯", "─".repeat(inner)))
    )?;
    Ok(())
}
