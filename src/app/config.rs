//! 配置模块

use std::net::SocketAddr;

/// 应用运行所需的全部配置
pub struct AppConfig {
    /// HTTP 服务监听地址
    pub bind_addr: SocketAddr,
    /// token 过期时间（天）
    pub token_expire: i64,
    /// PostgreSQL 数据库连接字符串
    pub database_url: String,
}

/// 从环境变量创建应用配置
/// 
/// # Panics
/// 发生以下情况时，应用将无法启动并发生 Panic：
/// * 环境变量 `SERVER_HOST` 或 `SERVER_PORT` 格式不正确，无法解析为有效的主机地址
/// * 环境变量 `TOKEN_EXPIRE` 包含非数字字符
/// * 未设置必须的环境变量 `DATABASE_URL`
pub fn from_env() -> AppConfig {
    // 尝试加载项目根目录下的 .env 文件
    dotenvy::dotenv().ok();

    // 读取服务主机地址
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    // 读取服务端口
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_owned());
    // 把主机和端口解析为 SocketAddr
    let bind_addr = format!("{host}:{port}").parse().expect("SERVER_HOST 或 SERVER_PORT 配置无效");

    // 读取 token 过期时间
    let token_expire = std::env::var("TOKEN_EXPIRE").unwrap_or_else(|_| "30".to_owned()).parse().expect("TOKEN_EXPIRE 必须是数字");

    // 读取数据库连接字符串
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL 必须设置");

    // 返回强类型应用配置
    AppConfig { bind_addr, token_expire, database_url }
}
