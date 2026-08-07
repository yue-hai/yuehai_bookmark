//! 全局共享的核心能力
//!
//! 此层存放与具体业务领域无关的错误处理、响应协议和应用状态

/// 导出全局错误定义及其 HTTP 响应转换实现
pub mod error;
/// 导出统一 API 成功响应的数据结构
pub mod response;
/// 导出由 Axum 注入给 Handler 的共享应用状态
pub mod state;
