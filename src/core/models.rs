use serde::{Deserialize, Serialize};
use std::fmt;

/// 订阅计划类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanType {
    Free,
    Plus,
    Pro,
    Team,
    Business,
    Enterprise,
    Edu,
    Unknown,
}

impl fmt::Display for PlanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanType::Free => write!(f, "free"),
            PlanType::Plus => write!(f, "plus"),
            PlanType::Pro => write!(f, "pro"),
            PlanType::Team => write!(f, "team"),
            PlanType::Business => write!(f, "business"),
            PlanType::Enterprise => write!(f, "enterprise"),
            PlanType::Edu => write!(f, "edu"),
            PlanType::Unknown => write!(f, "unknown"),
        }
    }
}

impl PlanType {
    /// 从字符串解析计划类型（大小写不敏感）
    pub fn from_str_loose(s: &str) -> PlanType {
        match s.to_lowercase().as_str() {
            "free" => PlanType::Free,
            "plus" => PlanType::Plus,
            "pro" => PlanType::Pro,
            "team" => PlanType::Team,
            "business" => PlanType::Business,
            "enterprise" => PlanType::Enterprise,
            "edu" => PlanType::Edu,
            _ => PlanType::Unknown,
        }
    }
}

/// 认证模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    Chatgpt,
    Apikey,
}

/// 速率限制窗口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitWindow {
    pub used_percent: f64,
    pub window_minutes: Option<i64>,
    pub resets_at: Option<i64>,
}

/// 积分快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditsSnapshot {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

/// 速率限制快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSnapshot {
    pub primary: Option<RateLimitWindow>,
    pub secondary: Option<RateLimitWindow>,
    pub credits: Option<CreditsSnapshot>,
    pub plan_type: Option<PlanType>,
}

/// 账号记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub email: String,
    pub alias: String,
    pub plan: Option<PlanType>,
    pub auth_mode: Option<AuthMode>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub last_usage: Option<RateLimitSnapshot>,
    pub last_usage_at: Option<i64>,
}

/// 注册表（与原始 registry.json 格式兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub active_email: Option<String>,
    pub accounts: Vec<AccountRecord>,
}

impl Registry {
    /// 创建空注册表
    pub fn new() -> Self {
        Registry {
            version: 2,
            active_email: None,
            accounts: Vec::new(),
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 auth.json 解析出的认证信息
#[derive(Debug)]
pub struct AuthInfo {
    pub email: Option<String>,
    pub plan: Option<PlanType>,
    pub auth_mode: AuthMode,
}

/// 导入摘要
#[derive(Debug, Default)]
pub struct ImportSummary {
    pub imported: usize,
    pub skipped: usize,
}

/// 根据速率限制窗口的分钟数查找对应窗口
pub fn resolve_rate_window(
    usage: &Option<RateLimitSnapshot>,
    minutes: i64,
    fallback_primary: bool,
) -> Option<&RateLimitWindow> {
    let usage = usage.as_ref()?;

    if let Some(ref p) = usage.primary {
        if p.window_minutes == Some(minutes) {
            return Some(p);
        }
    }
    if let Some(ref s) = usage.secondary {
        if s.window_minutes == Some(minutes) {
            return Some(s);
        }
    }

    if fallback_primary {
        usage.primary.as_ref()
    } else {
        usage.secondary.as_ref()
    }
}

/// 计算剩余百分比
pub fn remaining_percent(used_percent: f64) -> i64 {
    let remaining = 100.0 - used_percent;
    if remaining <= 0.0 {
        0
    } else if remaining >= 100.0 {
        100
    } else {
        remaining as i64
    }
}

/// 解析账号的实际计划类型（优先用 plan 字段，回退到 usage 中的 plan_type）
pub fn resolve_plan(rec: &AccountRecord) -> Option<&PlanType> {
    if rec.plan.is_some() {
        return rec.plan.as_ref();
    }
    if let Some(ref usage) = rec.last_usage {
        return usage.plan_type.as_ref();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_type_from_str() {
        assert_eq!(PlanType::from_str_loose("PLUS"), PlanType::Plus);
        assert_eq!(PlanType::from_str_loose("Pro"), PlanType::Pro);
        assert_eq!(PlanType::from_str_loose("unknown_value"), PlanType::Unknown);
    }

    #[test]
    fn test_remaining_percent() {
        assert_eq!(remaining_percent(30.0), 70);
        assert_eq!(remaining_percent(100.0), 0);
        assert_eq!(remaining_percent(0.0), 100);
        assert_eq!(remaining_percent(150.0), 0);
    }

    #[test]
    fn test_registry_default() {
        let reg = Registry::new();
        assert_eq!(reg.version, 2);
        assert!(reg.active_email.is_none());
        assert!(reg.accounts.is_empty());
    }
}
