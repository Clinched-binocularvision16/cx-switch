use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::models::*;

/// 扫描最新的会话日志，提取额度使用信息
pub fn scan_latest_usage(codex_home: &Path) -> Result<Option<RateLimitSnapshot>> {
    let sessions_root = codex_home.join("sessions");
    if !sessions_root.exists() {
        return Ok(None);
    }

    let mut latest_path: Option<PathBuf> = None;
    let mut latest_mtime: Option<std::time::SystemTime> = None;

    // 递归遍历 sessions 目录查找最新的 rollout-*.jsonl 文件
    walk_dir_for_rollout(&sessions_root, &mut latest_path, &mut latest_mtime)?;

    match latest_path {
        Some(path) => scan_file_for_usage(&path),
        None => Ok(None),
    }
}

/// 递归遍历目录查找 rollout 文件
fn walk_dir_for_rollout(
    dir: &Path,
    latest_path: &mut Option<PathBuf>,
    latest_mtime: &mut Option<std::time::SystemTime>,
) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_dir() {
            walk_dir_for_rollout(&entry.path(), latest_path, latest_mtime)?;
        } else if ft.is_file() || ft.is_symlink() {
            let name = entry.file_name().to_string_lossy().to_string();
            if is_rollout_file(&name) {
                if let Ok(meta) = entry.metadata() {
                    let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    let is_newer = match latest_mtime {
                        Some(prev) => mtime > *prev,
                        None => true,
                    };
                    if is_newer {
                        *latest_mtime = Some(mtime);
                        *latest_path = Some(entry.path());
                    }
                }
            }
        }
    }
    Ok(())
}

/// 判断是否为 rollout 日志文件
fn is_rollout_file(name: &str) -> bool {
    name.starts_with("rollout-") && name.ends_with(".jsonl")
}

/// 扫描单个文件中的额度使用信息（取最后一条有效数据）
fn scan_file_for_usage(path: &Path) -> Result<Option<RateLimitSnapshot>> {
    let data = fs::read_to_string(path)?;
    let mut last: Option<RateLimitSnapshot> = None;

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(snap) = parse_usage_line(trimmed) {
            last = Some(snap);
        }
    }

    Ok(last)
}

/// 解析单行 JSONL 数据，提取额度信息
pub fn parse_usage_line(line: &str) -> Option<RateLimitSnapshot> {
    let root: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = root.as_object()?;

    // 检查 type == "event_msg"
    let t = obj.get("type")?.as_str()?;
    if t != "event_msg" {
        return None;
    }

    // 检查 payload.type == "token_count"
    let payload = obj.get("payload")?.as_object()?;
    let ptype = payload.get("type")?.as_str()?;
    if ptype != "token_count" {
        return None;
    }

    // 提取 rate_limits
    let rate_limits = payload.get("rate_limits")?;
    parse_rate_limits(rate_limits)
}

/// 解析 rate_limits 对象
fn parse_rate_limits(v: &serde_json::Value) -> Option<RateLimitSnapshot> {
    let obj = v.as_object()?;
    let mut snap = RateLimitSnapshot {
        primary: None,
        secondary: None,
        credits: None,
        plan_type: None,
    };

    if let Some(p) = obj.get("primary") {
        snap.primary = parse_window(p);
    }
    if let Some(s) = obj.get("secondary") {
        snap.secondary = parse_window(s);
    }
    if let Some(c) = obj.get("credits") {
        snap.credits = parse_credits(c);
    }
    if let Some(p) = obj.get("plan_type").and_then(|v| v.as_str()) {
        snap.plan_type = Some(PlanType::from_str_loose(p));
    }

    Some(snap)
}

/// 解析速率限制窗口
fn parse_window(v: &serde_json::Value) -> Option<RateLimitWindow> {
    let obj = v.as_object()?;
    let used_percent = match obj.get("used_percent")? {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        _ => return None,
    };
    let window_minutes = obj.get("window_minutes").and_then(|v| v.as_i64());
    let resets_at = obj.get("resets_at").and_then(|v| v.as_i64());

    Some(RateLimitWindow {
        used_percent,
        window_minutes,
        resets_at,
    })
}

/// 解析积分信息
fn parse_credits(v: &serde_json::Value) -> Option<CreditsSnapshot> {
    let obj = v.as_object()?;
    Some(CreditsSnapshot {
        has_credits: obj.get("has_credits").and_then(|v| v.as_bool()).unwrap_or(false),
        unlimited: obj.get("unlimited").and_then(|v| v.as_bool()).unwrap_or(false),
        balance: obj.get("balance").and_then(|v| v.as_str()).map(|s| s.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_usage_line_valid() {
        let line = r#"{"type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":30.0,"window_minutes":300,"resets_at":1700000000},"secondary":{"used_percent":10.0,"window_minutes":10080,"resets_at":1700100000},"plan_type":"plus"}}}"#;
        let snap = parse_usage_line(line).unwrap();
        assert!(snap.primary.is_some());
        assert!(snap.secondary.is_some());
        assert_eq!(snap.primary.unwrap().used_percent, 30.0);
        assert_eq!(snap.plan_type.unwrap(), PlanType::Plus);
    }

    #[test]
    fn test_parse_usage_line_wrong_type() {
        let line = r#"{"type":"other","payload":{}}"#;
        assert!(parse_usage_line(line).is_none());
    }

    #[test]
    fn test_is_rollout_file() {
        assert!(is_rollout_file("rollout-abc.jsonl"));
        assert!(!is_rollout_file("other.jsonl"));
        assert!(!is_rollout_file("rollout-abc.json"));
    }
}
