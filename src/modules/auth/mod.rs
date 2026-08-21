//! 认证模块

/// HTTP 请求和响应结构
pub mod dto;
/// HTTP 请求提取器
pub mod extractors;
/// HTTP 入口
pub mod handlers;
/// 数据模型
pub mod model;
/// 数据库访问
pub mod repository;
/// 业务逻辑
pub mod service;

/// 注册 /api/auth 下的认证相关路由
pub mod routes;
