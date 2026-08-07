//! 面向 API 边界的数据传输对象（DTO）
//!
//! 请求 DTO 由 Serde 从 JSON 反序列化，不直接承载数据库行为

/// 导出创建书签请求 DTO。
pub mod create;
