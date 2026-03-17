/// 相对时间格式化
pub fn format_relative_time(ts: i64, now: i64) -> String {
    if ts <= 0 {
        return "-".to_string();
    }
    let delta = (now - ts).max(0);

    if delta < 60 {
        "Now".to_string()
    } else if delta < 3600 {
        format!("{}m ago", delta / 60)
    } else if delta < 86400 {
        format!("{}h ago", delta / 3600)
    } else {
        format!("{}d ago", delta / 86400)
    }
}

/// 相对时间格式化（支持 Option 值，None 返回 "-"）
pub fn format_relative_time_or_dash(ts: Option<i64>, now: i64) -> String {
    match ts {
        Some(t) if t > 0 => format_relative_time(t, now),
        _ => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        assert_eq!(format_relative_time(1000, 1000), "Now");
        assert_eq!(format_relative_time(1000, 1030), "Now");
    }

    #[test]
    fn test_minutes() {
        assert_eq!(format_relative_time(880, 1000), "2m ago");
    }

    #[test]
    fn test_hours() {
        let now = 1000 + 14 * 3600;
        assert_eq!(format_relative_time(1000, now), "14h ago");
    }

    #[test]
    fn test_days() {
        let now = 1000 + 24 * 3600;
        assert_eq!(format_relative_time(1000, now), "1d ago");
    }

    #[test]
    fn test_dash() {
        assert_eq!(format_relative_time_or_dash(None, 0), "-");
        assert_eq!(format_relative_time_or_dash(Some(0), 0), "-");
        assert_eq!(format_relative_time_or_dash(Some(-1), 0), "-");
    }
}
