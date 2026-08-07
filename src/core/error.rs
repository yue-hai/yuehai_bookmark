//! 定义 API 的统一错误响应格式
//!
//! Handler 返回 `Result<_, AppError>` 后，Axum 会调用本模块的 `IntoResponse` 实现
//! 避免每个接口重复构造状态码和 JSON 错误体

// 导入 Rust 标准库的 Cow 类型，用于在响应消息中高效地处理静态字符串和动态字符串
use std::borrow::Cow;
// 导入 HTTP 状态码、响应转换 trait 和 Axum JSON 响应包装器。
use axum::{Json, http::StatusCode, response::IntoResponse};
// 导入 thiserror 宏，用于简化错误类型的定义
use thiserror::Error;
// 导入当前项目的统一 API 响应结构体
use crate::core::response::ApiResponse;

/// 应用层向外暴露的统一错误枚举
#[derive(Error, Debug)]
pub enum AppError {
    /// 401 未授权访问
    #[error("当前请求未授权，请检查 Token")]
    Unauthorized,
    /// 404 资源未找到
    #[error("未找到指定的资源: {0}")]
    NotFound(String),
    /// 500 数据库错误，包含底层 sqlx 错误信息
    #[error("数据库底层运行异常")]
    DatabaseError(#[from] sqlx::Error),
}

/// 告诉 Axum 如何把 AppError 转换为一个完整 HTTP 响应。
impl IntoResponse for AppError {
    /// 消费错误对象，并生成状态码与 JSON 响应体。
    fn into_response(self) -> axum::response::Response {
        // 根据具体错误类型计算 HTTP 状态码和对外错误消息。
        let status = match &self {
            // 401 未授权访问
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            // 404 资源未找到
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            // 500 数据库错误，包含底层 sqlx 错误信息
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // 按项目统一协议构造错误响应 JSON
        let response = ApiResponse::<()> {
            // 业务状态码
            code: status.as_u16(),
            // 供客户端展示或调试的响应说明
            msg: Cow::Owned(self.to_string()),
            // 错误场景没有业务数据，因此明确返回 None
            data: None,
        };

        // 将状态码和 JSON body 组合为 Axum 可发送的 HTTP 响应
        (status, Json(response)).into_response()
    }
}
