use anyhow::Result;
use crate::core::registry;
use crate::tui::multi_selector;

/// 执行 remove 命令
pub fn execute() -> Result<()> {
    let codex_home = registry::resolve_codex_home()?;
    let mut reg = registry::load_registry(&codex_home)?;

    if reg.accounts.is_empty() {
        println!("暂无已管理的账号。");
        return Ok(());
    }

    // 交互式多选
    let selected = multi_selector::select_accounts_to_remove(&reg)?;

    let indices = match selected {
        Some(idx) if !idx.is_empty() => idx,
        _ => {
            println!("已取消删除操作。");
            return Ok(());
        }
    };

    // 收集要删除的邮箱列表
    let emails: Vec<String> = indices
        .iter()
        .map(|&i| reg.accounts[i].email.clone())
        .collect();

    // 执行删除
    registry::remove_accounts(&codex_home, &mut reg, &indices)?;

    // 如果活跃账号被删除，选择替代
    if reg.active_email.is_none() && !reg.accounts.is_empty() {
        if let Some(best_idx) = registry::select_best_account_index_by_usage(&reg) {
            let best_email = reg.accounts[best_idx].email.clone();
            let new_path = registry::account_auth_path(&codex_home, &best_email);
            let active_path = registry::active_auth_path(&codex_home);
            if new_path.exists() {
                registry::copy_file(&new_path, &active_path)?;
                registry::set_active_account(&mut reg, &best_email);
                println!("⟳ 自动切换到: {}", best_email);
            }
        }
    }

    registry::save_registry(&codex_home, &mut reg)?;

    for email in &emails {
        println!("✗ 已删除: {}", email);
    }

    Ok(())
}
