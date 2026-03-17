use crossterm::style::{Attribute, Color, Stylize};

/// 根据额度剩余百分比返回对应颜色
pub fn usage_color(remaining_percent: i64) -> Color {
    if remaining_percent >= 50 {
        Color::Green
    } else if remaining_percent >= 20 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// 计划类型对应的颜色
pub fn plan_color(plan: &str) -> Color {
    match plan {
        "pro" => Color::Magenta,
        "plus" => Color::Green,
        "team" => Color::Blue,
        "business" => Color::Cyan,
        "enterprise" => Color::Yellow,
        "edu" => Color::DarkCyan,
        "free" => Color::DarkGrey,
        _ => Color::Grey,
    }
}

/// 活跃账号样式
pub fn active_style(text: &str) -> String {
    text.with(Color::Green).attribute(Attribute::Bold).to_string()
}

/// 选中项样式
pub fn selected_style(text: &str) -> String {
    text.with(Color::Green).attribute(Attribute::Bold).to_string()
}

/// 暗淡样式
pub fn dim_style(text: &str) -> String {
    text.with(Color::DarkGrey).to_string()
}

/// 警告样式
pub fn warn_style(text: &str) -> String {
    text.with(Color::Yellow).attribute(Attribute::Bold).to_string()
}

/// 错误样式
pub fn error_style(text: &str) -> String {
    text.with(Color::Red).attribute(Attribute::Bold).to_string()
}

/// 表头样式
pub fn header_style(text: &str) -> String {
    text.attribute(Attribute::Bold)
        .attribute(Attribute::Underlined)
        .to_string()
}

/// 生成迷你进度条（10 字符宽）
pub fn mini_progress_bar(remaining_percent: i64) -> String {
    let width = 10;
    let filled = ((remaining_percent as f64 / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;

    let bar_str = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(empty)
    );

    let color = usage_color(remaining_percent);
    bar_str.with(color).to_string()
}

/// 生成带颜色的百分比文字
pub fn colored_percent(remaining_percent: i64) -> String {
    let text = format!("{}%", remaining_percent);
    let color = usage_color(remaining_percent);
    text.with(color).to_string()
}

/// 检测终端是否支持颜色
pub fn color_enabled() -> bool {
    atty_stdout()
}

/// 检测标准输出是否为 TTY
fn atty_stdout() -> bool {
    crossterm::tty::IsTty::is_tty(&std::io::stdout())
}

/// 获取终端宽度
pub fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
}
