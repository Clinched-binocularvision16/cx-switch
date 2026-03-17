use anyhow::Result;
use crate::core::registry;

/// 执行 import 命令
pub fn execute(path: &str, alias: Option<&str>) -> Result<()> {
    let codex_home = registry::resolve_codex_home()?;
    let mut reg = registry::load_registry(&codex_home)?;

    let summary = registry::import_auth_path(&codex_home, &mut reg, path, alias)?;

    registry::save_registry(&codex_home, &mut reg)?;

    if summary.imported > 0 {
        println!("✓ 已导入 {} 个账号。", summary.imported);
    }
    if summary.skipped > 0 {
        println!("⚠ 跳过 {} 个文件（无效或重复）。", summary.skipped);
    }
    if summary.imported == 0 && summary.skipped == 0 {
        println!("未找到可导入的认证文件。");
    }

    Ok(())
}
