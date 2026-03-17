use crate::core::models::*;
use crate::tui::{icons, theme};
use crate::utils::timefmt;
use chrono::prelude::*;
use std::io::{self, Write};

/// 以增强表格形式打印账号列表
pub fn print_accounts_table(reg: &Registry) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let use_color = theme::color_enabled();
    let now = crate::core::registry::now_timestamp();
    let term_width = theme::terminal_width();

    // 列标题
    let headers = ["EMAIL", "PLAN", "5H USAGE", "WEEKLY USAGE", "LAST ACTIVITY"];
    let mut widths: [usize; 5] = [
        headers[0].len(),
        headers[1].len(),
        headers[2].len(),
        headers[3].len(),
        headers[4].len(),
    ];

    // 预计算每行数据并确定列宽
    let mut rows: Vec<RowData> = Vec::new();
    for rec in &reg.accounts {
        let email_cell = format_email_cell(rec);
        let plan = resolve_plan(rec)
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());
        let rate_5h = resolve_rate_window(&rec.last_usage, 300, true);
        let rate_week = resolve_rate_window(&rec.last_usage, 10080, false);
        let rate_5h_str = format_rate_limit_full(rate_5h, now);
        let rate_week_str = format_rate_limit_full(rate_week, now);
        let last = timefmt::format_relative_time_or_dash(rec.last_usage_at, now);
        let is_active = reg.active_email.as_deref() == Some(&rec.email);

        widths[0] = widths[0].max(email_cell.len());
        widths[1] = widths[1].max(plan.len());
        widths[2] = widths[2].max(rate_5h_str.len());
        widths[3] = widths[3].max(rate_week_str.len());
        widths[4] = widths[4].max(last.len());

        rows.push(RowData {
            email_cell,
            plan,
            rate_5h_str,
            rate_week_str,
            last,
            is_active,
            rate_5h_remaining: rate_5h.map(|w| remaining_percent(w.used_percent)),
            rate_week_remaining: rate_week.map(|w| remaining_percent(w.used_percent)),
        });
    }

    // 调整列宽以适配终端
    adjust_widths(&mut widths, term_width);

    // 打印分隔线
    let total_width: usize = 2 + widths.iter().sum::<usize>() + 2 * (widths.len() - 1);
    let separator: String = icons::BOX_HORIZONTAL.repeat(total_width);

    // 打印表头
    if use_color {
        write!(out, "  ")?;
        write!(out, "{}", theme::header_style(&truncate(&headers[0], widths[0])))?;
        write!(out, "  ")?;
        write!(out, "{}", theme::header_style(&truncate(&headers[1], widths[1])))?;
        write!(out, "  ")?;
        let h5 = if widths[2] >= 8 { "5H USAGE" } else { "5H" };
        write!(out, "{}", theme::header_style(&truncate(h5, widths[2])))?;
        write!(out, "  ")?;
        let hw = if widths[3] >= 12 {
            "WEEKLY USAGE"
        } else if widths[3] >= 6 {
            "WEEKLY"
        } else {
            "W"
        };
        write!(out, "{}", theme::header_style(&truncate(hw, widths[3])))?;
        write!(out, "  ")?;
        let hl = if widths[4] >= 13 {
            "LAST ACTIVITY"
        } else {
            "LAST"
        };
        write!(out, "{}", theme::header_style(&truncate(hl, widths[4])))?;
        writeln!(out)?;
    } else {
        write!(out, "  ")?;
        write_padded(&mut out, &headers[0], widths[0])?;
        write!(out, "  ")?;
        write_padded(&mut out, &headers[1], widths[1])?;
        write!(out, "  ")?;
        write_padded(&mut out, "5H USAGE", widths[2])?;
        write!(out, "  ")?;
        write_padded(&mut out, "WEEKLY USAGE", widths[3])?;
        write!(out, "  ")?;
        write_padded(&mut out, "LAST ACTIVITY", widths[4])?;
        writeln!(out)?;
    }

    // 分隔线
    if use_color {
        writeln!(out, "{}", theme::dim_style(&separator))?;
    } else {
        writeln!(out, "{}", separator)?;
    }

    // 打印每行数据
    for row in &rows {
        let prefix = if row.is_active {
            format!("{} ", icons::ACTIVE)
        } else {
            "  ".to_string()
        };

        if use_color {
            if row.is_active {
                write!(out, "{}", theme::active_style(&prefix))?;
            } else {
                write!(out, "{}", theme::dim_style(&prefix))?;
            }
        } else {
            write!(out, "{}", if row.is_active { "* " } else { "  " })?;
        }

        let email_cell = truncate(&row.email_cell, widths[0]);
        let plan_cell = truncate(&row.plan, widths[1]);
        let rate_5h_cell = truncate(&row.rate_5h_str, widths[2]);
        let rate_week_cell = truncate(&row.rate_week_str, widths[3]);
        let last_cell = truncate(&row.last, widths[4]);

        if use_color {
            if row.is_active {
                write!(out, "{}", theme::active_style(&pad_right(&email_cell, widths[0])))?;
            } else {
                write!(out, "{}", theme::dim_style(&pad_right(&email_cell, widths[0])))?;
            }
            write!(out, "  ")?;

            // 计划类型着色
            let plan_padded = pad_right(&plan_cell, widths[1]);
            let plan_colored = crossterm::style::Stylize::with(
                plan_padded.as_str(),
                theme::plan_color(&row.plan),
            );
            write!(out, "{}", plan_colored)?;
            write!(out, "  ")?;

            write_padded(&mut out, &rate_5h_cell, widths[2])?;
            write!(out, "  ")?;
            write_padded(&mut out, &rate_week_cell, widths[3])?;
            write!(out, "  ")?;
            write_padded(&mut out, &last_cell, widths[4])?;
        } else {
            write_padded(&mut out, &email_cell, widths[0])?;
            write!(out, "  ")?;
            write_padded(&mut out, &plan_cell, widths[1])?;
            write!(out, "  ")?;
            write_padded(&mut out, &rate_5h_cell, widths[2])?;
            write!(out, "  ")?;
            write_padded(&mut out, &rate_week_cell, widths[3])?;
            write!(out, "  ")?;
            write_padded(&mut out, &last_cell, widths[4])?;
        }
        writeln!(out)?;
    }

    out.flush()?;
    Ok(())
}

#[allow(dead_code)]
struct RowData {
    email_cell: String,
    plan: String,
    rate_5h_str: String,
    rate_week_str: String,
    last: String,
    is_active: bool,
    rate_5h_remaining: Option<i64>,
    rate_week_remaining: Option<i64>,
}

/// 格式化邮箱单元格（包含别名）
fn format_email_cell(rec: &AccountRecord) -> String {
    if rec.alias.is_empty() {
        rec.email.clone()
    } else {
        format!("({}){}", rec.alias, rec.email)
    }
}

/// 格式化速率限制完整信息
fn format_rate_limit_full(window: Option<&RateLimitWindow>, now: i64) -> String {
    let window = match window {
        Some(w) => w,
        None => return "-".to_string(),
    };
    let reset_at = match window.resets_at {
        Some(r) => r,
        None => return "-".to_string(),
    };

    if now >= reset_at {
        return "100% -".to_string();
    }

    let remaining = remaining_percent(window.used_percent);
    let time_str = format_reset_time(reset_at, now);
    format!("{}% ({})", remaining, time_str)
}

/// 格式化重置时间
fn format_reset_time(reset_at: i64, now: i64) -> String {
    let reset_dt = match Local.timestamp_opt(reset_at, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => return "-".to_string(),
    };
    let now_dt = match Local.timestamp_opt(now, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => return "-".to_string(),
    };

    let same_day = reset_dt.date_naive() == now_dt.date_naive();
    if same_day {
        reset_dt.format("%H:%M").to_string()
    } else {
        reset_dt.format("%H:%M on %d %b").to_string()
    }
}

/// 右填充字符串到指定宽度
fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

/// 写入右填充内容
fn write_padded(out: &mut impl Write, value: &str, width: usize) -> io::Result<()> {
    write!(out, "{}", value)?;
    if value.len() < width {
        write!(out, "{}", " ".repeat(width - value.len()))?;
    }
    Ok(())
}

/// 截断字符串
fn truncate(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }
    if max_len == 0 {
        return String::new();
    }
    if max_len == 1 {
        return ".".to_string();
    }
    format!("{}.", &value[..max_len - 1])
}

/// 调整列宽以适配终端宽度
fn adjust_widths(widths: &mut [usize; 5], term_width: usize) {
    if term_width == 0 {
        return;
    }
    let total: usize = 2 + widths.iter().sum::<usize>() + 2 * (widths.len() - 1);
    if total <= term_width {
        return;
    }

    let min_widths = [10usize, 4, 1, 1, 4];
    let mut over = total - term_width;

    for (i, min) in min_widths.iter().enumerate() {
        if over == 0 {
            break;
        }
        if widths[i] > *min {
            let reducible = widths[i] - *min;
            let reduce = reducible.min(over);
            widths[i] -= reduce;
            over -= reduce;
        }
    }
}
