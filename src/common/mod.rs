//! 跨业务模块、长期稳定复用的通用能力

/// 与具体业务无关的 HTTP 协议实现
pub mod http;

/// 全局错误定义及其 HTTP 响应转换实现
pub mod error;
