use anyhow::Result;
use crate::core::{registry, sessions};
use crate::tui::selector;

/// 执行 switch 命令
pub fn execute(email_filter: Option<String>) -> Result<()> {
    let codex_home = registry::resolve_codex_home()?;
    let mut reg = registry::load_registry(&codex_home)?;

    if reg.accounts.is_empty() {
        println!("暂无已管理的账号。");
        println!("使用 `cx-switch login` 添加当前账号。");
        return Ok(());
    }

    // 同步活跃账号
    let _ = registry::sync_active_account_from_auth(&codex_home, &mut reg)?;

    // 从认证文件刷新 plan 信息
    registry::refresh_accounts_from_auth(&codex_home, &mut reg)?;

    // 扫描会话日志更新额度
    if let Some(active) = reg.active_email.clone() {
        if let Some(snapshot) = sessions::scan_latest_usage(&codex_home)? {
            registry::update_usage(&mut reg, &active, snapshot);
        }
    }

    // 确定要切换到的邮箱
    let target_email = match email_filter {
        Some(filter) => {
            // 模糊匹配
            let matches: Vec<usize> = reg
                .accounts
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    r.email.contains(&filter)
                        || r.alias.contains(&filter)
                })
                .map(|(i, _)| i)
                .collect();

            match matches.len() {
                0 => {
                    eprintln!("未找到匹配 \"{}\" 的账号。", filter);
                    return Ok(());
                }
                1 => Some(reg.accounts[matches[0]].email.clone()),
                _ => {
                    // 多个匹配：交互式选择
                    selector::select_from_indices(&reg, &matches)?
                }
            }
        }
        None => {
            // 无过滤参数：交互式选择
            selector::select_account(&reg)?
        }
    };

    let target_email = match target_email {
        Some(e) => e,
        None => {
            println!("已取消切换。");
            return Ok(());
        }
    };

    // 检查是否已经是活跃账号
    if reg.active_email.as_deref() == Some(&target_email) {
        println!("✓ {} 已经是活跃账号。", target_email);
        return Ok(());
    }

    // 执行切换
    let active_path = registry::active_auth_path(&codex_home);
    let new_path = registry::account_auth_path(&codex_home, &target_email);

    if !new_path.exists() {
        anyhow::bail!(
            "找不到账号 {} 的认证文件: {}",
            target_email,
            new_path.display()
        );
    }

    // 备份当前 auth.json
    if active_path.exists() {
        registry::backup_auth_if_changed(&codex_home, &active_path, &new_path)?;
    }

    // 复制新认证文件为活跃
    registry::copy_file(&new_path, &active_path)?;

    // 更新注册表
    registry::set_active_account(&mut reg, &target_email);
    registry::save_registry(&codex_home, &mut reg)?;

    println!("✓ 已切换到: {}", target_email);

    Ok(())
}
