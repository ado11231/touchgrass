//! Arrange dates into Monday-based heatmap columns.

use super::*;

/// Number of week columns in the sliding view.
const WINDOW_WEEKS: i64 = 52;
/// Days in a complete heatmap column.
const WEEK_DAYS: i64 = 7;

fn monday(date: NaiveDate) -> Result<NaiveDate> {
    date.checked_sub_signed(Duration::days(i64::from(
        date.weekday().num_days_from_monday(),
    )))
    .context("heatmap date is outside the supported range")
}

/// Write the date grid in panels that fit the terminal.
pub(super) fn render(
    output: &mut String,
    days: &BTreeMap<NaiveDate, BTreeMap<String, u64>>,
    installed_on: Option<NaiveDate>,
    today: NaiveDate,
    year: Option<i32>,
    width: usize,
    color: Color,
) -> Result<()> {
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
        let earliest = monday(today)?
            .checked_sub_signed(Duration::weeks(WINDOW_WEEKS - 1))
            .context("invalid heatmap range")?;
        let installed = installed_on
            .or_else(|| days.keys().next().copied())
            .unwrap_or(today)
            .min(today);
        vec![(monday(installed)?.max(earliest), today)]
    };
    for (start, end) in ranges {
        let mut first = monday(start)?;
        while first <= end {
            let remaining = (end - first).num_days() / WEEK_DAYS + 1;
            let count = columns.min(remaining as usize);
            panel(output, days, (start, end, today, first), count, color)?;
            first = first
                .checked_add_signed(Duration::weeks(count as i64))
                .context("invalid heatmap range")?;
            if first <= end {
                output.push('\n');
            }
        }
    }
    Ok(())
}

fn panel(
    output: &mut String,
    days: &BTreeMap<NaiveDate, BTreeMap<String, u64>>,
    dates: (NaiveDate, NaiveDate, NaiveDate, NaiveDate),
    columns: usize,
    color: Color,
) -> Result<()> {
    let (start, end, today, first) = dates;
    let mut labels = vec![' '; columns * COLUMN_WIDTH + 3];
    let mut next_label = 0;
    for column in 0..columns {
        let date = first + Duration::weeks(column as i64);
        let last_day = (date + Duration::days(WEEK_DAYS - 1)).min(end);
        if column == 0 || date.month() != last_day.month() || date.day() == 1 {
            let position = column * COLUMN_WIDTH;
            if position >= next_label {
                let label = last_day.format("%b").to_string();
                for (offset, character) in label.chars().enumerate() {
                    labels[position + offset] = character;
                }
                next_label = position + label.len() + 1;
            }
        }
    }
    writeln!(
        output,
        "         {}",
        labels.into_iter().collect::<String>().trim_end()
    )?;
    for weekday in 0..WEEK_DAYS {
        let label = match weekday {
            0 => "Mon",
            2 => "Wed",
            4 => "Fri",
            _ => "   ",
        };
        write!(output, "  {label}    ")?;
        for column in 0..columns {
            let date = first + Duration::days(column as i64 * WEEK_DAYS + weekday);
            if date < start || date > end {
                output.push_str("  ");
                continue;
            }
            let seconds = if date > today {
                0
            } else {
                days.get(&date)
                    .map(|tools| tools.values().map(|seconds| u128::from(*seconds)).sum())
                    .unwrap_or(0)
            };
            let level = level(seconds);
            let square = match color {
                Color::None => SHADES[level].to_string(),
                _ => String::from("■"),
            };
            write!(output, "{} ", paint(&square, level, color))?;
        }
        output.push('\n');
    }
    Ok(())
}
