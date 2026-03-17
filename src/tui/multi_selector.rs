use crate::core::models::*;
use crate::tui::{icons, theme};
use crate::utils::timefmt;
use crossterm::{
    cursor, event,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};

/// 交互式多选账号（返回选中的索引列表）
pub fn select_accounts_to_remove(reg: &Registry) -> anyhow::Result<Option<Vec<usize>>> {
    if reg.accounts.is_empty() {
        return Ok(None);
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    terminal::enable_raw_mode()?;
    let _cleanup = RawModeGuard;

    let now = crate::core::registry::now_timestamp();
    let mut cursor_idx: usize = 0;
    let mut checked = vec![false; reg.accounts.len()];

    loop {
        out.execute(terminal::Clear(ClearType::All))?;
        out.execute(cursor::MoveTo(0, 0))?;

        writeln!(out, "  选择要删除的账号：\r")?;
        writeln!(out, "\r")?;

        render_remove_list(&mut out, reg, cursor_idx, &checked, now)?;

        writeln!(out, "\r")?;
        writeln!(
            out,
            "  {}",
            theme::dim_style("↑/↓ 或 j/k 移动 · Space 切换选中 · Enter 确认删除 · Esc 退出")
        )?;
        out.flush()?;

        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Up | event::KeyCode::Char('k') => {
                    if cursor_idx > 0 {
                        cursor_idx -= 1;
                    }
                }
                event::KeyCode::Down | event::KeyCode::Char('j') => {
                    if cursor_idx + 1 < reg.accounts.len() {
                        cursor_idx += 1;
                    }
                }
                event::KeyCode::Char(' ') => {
                    checked[cursor_idx] = !checked[cursor_idx];
                }
                event::KeyCode::Enter => {
                    let selected: Vec<usize> =
                        checked.iter().enumerate()
                            .filter(|(_, &c)| c)
                            .map(|(i, _)| i)
                            .collect();
                    if selected.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(selected));
                }
                event::KeyCode::Esc => {
                    return Ok(None);
                }
                event::KeyCode::Char(c) if c.is_ascii_digit() => {
                    let num = c.to_digit(10).unwrap_or(0) as usize;
                    if num >= 1 && num <= reg.accounts.len() {
                        cursor_idx = num - 1;
                    }
                }
                _ => {}
            }
        }
    }
}

/// 渲染删除选择列表
fn render_remove_list(
    out: &mut impl Write,
    reg: &Registry,
    cursor_idx: usize,
    checked: &[bool],
    now: i64,
) -> io::Result<()> {
    for (i, rec) in reg.accounts.iter().enumerate() {
        let is_cursor = i == cursor_idx;
        let is_checked = checked[i];
        let is_active = reg.active_email.as_deref() == Some(&rec.email);

        let pointer = if is_cursor { icons::POINTER } else { " " };
        let checkbox = if is_checked {
            icons::CHECKBOX_ON
        } else {
            icons::CHECKBOX_OFF
        };

        let plan = resolve_plan(rec)
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());

        let last = timefmt::format_relative_time_or_dash(rec.last_usage_at, now);

        let line = format!(
            "  {} {} {:2} {:<30} {:<6} {}{}",
            pointer,
            checkbox,
            i + 1,
            &rec.email,
            plan,
            last,
            if is_active { "  [ACTIVE]" } else { "" }
        );

        if is_cursor {
            write!(out, "{}", theme::selected_style(&line))?;
        } else if is_checked {
            write!(out, "{}", theme::error_style(&line))?;
        } else if is_active {
            write!(out, "{}", theme::active_style(&line))?;
        } else {
            write!(out, "{}", theme::dim_style(&line))?;
        }
        writeln!(out, "\r")?;
    }

    Ok(())
}

/// RAII 守卫：退出时恢复终端
struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = io::stdout();
        let _ = out.execute(terminal::Clear(ClearType::All));
        let _ = out.execute(cursor::MoveTo(0, 0));
    }
}
