use crate::core::models::*;
use crate::tui::{icons, theme};
use crate::utils::timefmt;
use chrono::TimeZone;
use crossterm::{
    cursor, event,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};

/// 交互式单选账号（返回选中的邮箱）
pub fn select_account(reg: &Registry) -> anyhow::Result<Option<String>> {
    if reg.accounts.is_empty() {
        return Ok(None);
    }

    let indices: Vec<usize> = (0..reg.accounts.len()).collect();
    select_from_indices(reg, &indices)
}

/// 从指定索引列表中交互式选择（用于模糊匹配多结果场景）
pub fn select_from_indices(reg: &Registry, indices: &[usize]) -> anyhow::Result<Option<String>> {
    if indices.is_empty() {
        return Ok(None);
    }
    if indices.len() == 1 {
        return Ok(Some(reg.accounts[indices[0]].email.clone()));
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    // 进入 raw 模式
    terminal::enable_raw_mode()?;
    let _cleanup = RawModeGuard;

    let now = crate::core::registry::now_timestamp();
    let active_idx = find_active_index(reg, indices);
    let mut cursor_idx = active_idx.unwrap_or(0);

    loop {
        // 清屏并渲染列表
        out.execute(terminal::Clear(ClearType::All))?;
        out.execute(cursor::MoveTo(0, 0))?;

        writeln!(out, "  选择要切换的账号：\r")?;
        writeln!(out, "\r")?;

        render_select_list(&mut out, reg, indices, cursor_idx, active_idx, now)?;

        writeln!(out, "\r")?;
        writeln!(
            out,
            "  {}",
            theme::dim_style("↑/↓ 或 j/k 移动 · Enter 选择 · 1-9 跳转 · Esc 退出")
        )?;
        out.flush()?;

        // 读取按键
        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Up | event::KeyCode::Char('k') => {
                    if cursor_idx > 0 {
                        cursor_idx -= 1;
                    }
                }
                event::KeyCode::Down | event::KeyCode::Char('j') => {
                    if cursor_idx + 1 < indices.len() {
                        cursor_idx += 1;
                    }
                }
                event::KeyCode::Enter => {
                    return Ok(Some(reg.accounts[indices[cursor_idx]].email.clone()));
                }
                event::KeyCode::Esc => {
                    return Ok(None);
                }
                event::KeyCode::Char(c) if c.is_ascii_digit() => {
                    let num = c.to_digit(10).unwrap_or(0) as usize;
                    if num >= 1 && num <= indices.len() {
                        cursor_idx = num - 1;
                    }
                }
                _ => {}
            }
        }
    }
}

/// 渲染选择列表
fn render_select_list(
    out: &mut impl Write,
    reg: &Registry,
    indices: &[usize],
    cursor_idx: usize,
    active_idx: Option<usize>,
    now: i64,
) -> io::Result<()> {
    // 表头
    write!(out, "     ")?;
    write!(out, "{}", theme::header_style("EMAIL"))?;
    write!(out, "                          ")?;
    write!(out, "{}", theme::header_style("PLAN"))?;
    write!(out, "    ")?;
    write!(out, "{}", theme::header_style("5H"))?;
    write!(out, "             ")?;
    write!(out, "{}", theme::header_style("WEEKLY"))?;
    write!(out, "           ")?;
    write!(out, "{}", theme::header_style("LAST"))?;
    writeln!(out, "\r")?;

    for (pos, &idx) in indices.iter().enumerate() {
        let rec = &reg.accounts[idx];
        let is_cursor = pos == cursor_idx;
        let is_active = active_idx == Some(pos);

        // 前缀指示符
        let pointer = if is_cursor { icons::POINTER } else { " " };
        let active_mark = if is_active { icons::ACTIVE } else { " " };

        let plan = resolve_plan(rec)
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());

        let rate_5h = resolve_rate_window(&rec.last_usage, 300, true);
        let rate_week = resolve_rate_window(&rec.last_usage, 10080, false);
        let _rem_5h = rate_5h.map(|w| remaining_percent(w.used_percent));
        let _rem_week = rate_week.map(|w| remaining_percent(w.used_percent));

        let rate_5h_str = format_rate_short(rate_5h, now);
        let rate_week_str = format_rate_short(rate_week, now);
        let last = timefmt::format_relative_time_or_dash(rec.last_usage_at, now);

        // 构建行文本
        let line = format!(
            "  {} {} {:2} {:<28} {:<6} {:<14} {:<14} {}{}",
            pointer,
            active_mark,
            pos + 1,
            truncate_str(&rec.email, 28),
            plan,
            rate_5h_str,
            rate_week_str,
            last,
            if is_active { "  [ACTIVE]" } else { "" }
        );

        if is_cursor {
            write!(out, "{}", theme::selected_style(&line))?;
        } else if is_active {
            write!(out, "{}", theme::active_style(&line))?;
        } else {
            write!(out, "{}", theme::dim_style(&line))?;
        }
        writeln!(out, "\r")?;
    }

    Ok(())
}

/// 格式化简短速率信息（用于选择列表）
fn format_rate_short(window: Option<&RateLimitWindow>, now: i64) -> String {
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
    let reset_dt = chrono::Local
        .timestamp_opt(reset_at, 0)
        .single()
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|| "-".to_string());
    format!("{}% ({})", remaining, reset_dt)
}

/// 找到活跃账号在索引列表中的位置
fn find_active_index(reg: &Registry, indices: &[usize]) -> Option<usize> {
    let active = reg.active_email.as_deref()?;
    indices
        .iter()
        .position(|&idx| reg.accounts[idx].email == active)
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max <= 1 {
        ".".to_string()
    } else {
        format!("{}.", &s[..max - 1])
    }
}

/// RAII 守卫：退出时恢复终端
struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        // 清屏恢复
        let mut out = io::stdout();
        let _ = out.execute(terminal::Clear(ClearType::All));
        let _ = out.execute(cursor::MoveTo(0, 0));
    }
}
