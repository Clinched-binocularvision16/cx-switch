use anyhow::Result;
use crate::core::{auth, registry};
use std::process::Command;

/// 执行 login 命令
pub fn execute(skip: bool) -> Result<()> {
    let codex_home = registry::resolve_codex_home()?;

    if !skip {
        // 运行 codex login
        println!("正在运行 codex login...");
        let status = Command::new("codex")
            .arg("login")
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("codex login 成功。");
            }
            Ok(s) => {
                eprintln!("codex login 退出码: {:?}", s.code());
                eprintln!("提示: 如果你已经登录了，可以使用 `cx-switch login --skip` 跳过登录直接导入。");
                return Ok(());
            }
            Err(e) => {
                eprintln!("无法运行 codex login: {}", e);
                eprintln!("提示: 使用 `cx-switch login --skip` 跳过登录直接导入当前 auth.json。");
                return Ok(());
            }
        }
    }

    // 读取 auth.json
    let auth_path = registry::active_auth_path(&codex_home);
    if !auth_path.exists() {
        anyhow::bail!(
            "未找到认证文件: {}\n请先运行 `codex login` 或 `cx-switch login` 进行登录。",
            auth_path.display()
        );
    }

    let info = auth::parse_auth_info(auth_path.to_str().unwrap_or_default())?;
    let email = match &info.email {
        Some(e) => e.clone(),
        None => {
            anyhow::bail!("认证文件中缺少邮箱信息，可能不是有效的 Codex 认证文件。");
        }
    };

    // 加载注册表
    let mut reg = registry::load_registry(&codex_home)?;

    // 复制认证文件到 accounts 目录
    let dest = registry::account_auth_path(&codex_home, &email);
    registry::ensure_accounts_dir(&codex_home)?;
    registry::copy_file(&auth_path, &dest)?;

    // 创建并添加账号记录
    let record = registry::account_from_auth("", &info)?;
    let plan_str = info
        .plan
        .as_ref()
        .map(|p| p.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 检查是否已存在
    let exists = reg.accounts.iter().any(|r| r.email == email);
    registry::upsert_account(&mut reg, record);
    registry::set_active_account(&mut reg, &email);
    registry::save_registry(&codex_home, &mut reg)?;

    if exists {
        println!("✓ 已更新账号: {} ({})", email, plan_str);
    } else {
        println!("✓ 已添加账号: {} ({})", email, plan_str);
    }

    Ok(())
}
