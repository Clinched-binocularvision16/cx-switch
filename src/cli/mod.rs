pub mod commands;

use clap::{Parser, Subcommand};

/// 本地 Codex 多账号切换工具
#[derive(Parser)]
#[command(name = "cx-switch", version, about = "本地 Codex 多账号切换工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 列出所有账号及额度信息
    List,

    /// 登录并添加当前账号
    Login {
        /// 跳过 codex login，直接读取本地认证文件
        #[arg(long)]
        skip: bool,
    },

    /// 切换活跃账号
    Switch {
        /// 邮箱或邮箱片段（模糊匹配）
        email: Option<String>,
    },

    /// 导入认证文件或目录
    Import {
        /// 认证文件路径或目录路径
        path: String,

        /// 为导入的账号设置别名
        #[arg(long)]
        alias: Option<String>,
    },

    /// 移除一个或多个账号
    Remove,

    /// 额度监控守护进程
    Watch {
        /// 检查间隔（秒）
        #[arg(long, default_value = "60")]
        interval: u64,

        /// 低额度阈值（百分比）
        #[arg(long, default_value = "20")]
        threshold: u64,

        /// 额度不足时自动切换到最佳账号
        #[arg(long)]
        auto_switch: bool,
    },

    /// 登录并添加当前账号（废弃别名，请使用 login）
    #[command(hide = true)]
    Add {
        /// 跳过 codex login
        #[arg(long)]
        skip: bool,

        /// 废弃参数，等同于 --skip
        #[arg(long, hide = true)]
        no_login: bool,
    },
}
