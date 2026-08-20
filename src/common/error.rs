//! 全局错误定义及其 HTTP 响应转换实现
//!
//! Handler 返回 `Result<_, AppError>` 后，Axum 会调用本模块的 `IntoResponse` 实现
//! 避免每个接口重复构造状态码和 JSON 错误体

use crate::common::http::response::ApiResponse;
use axum::{Json, http::StatusCode, response::IntoResponse};
use std::borrow::Cow;
use thiserror::Error;

/// 应用层向外暴露的统一错误枚举
#[derive(Error, Debug)]
pub enum AppError {
    /// 400 请求参数错误
    #[error("请求参数无效：{0}")]
    BadRequest(&'static str),
    /// 401 统一凭据错误
    #[error("邮箱或密码错误")]
    InvalidCredentials,
    /// 401 未授权访问
    #[error("当前请求未授权，请检查 Token")]
    Unauthorized,
    /// 404 资源未找到
    #[error("未找到指定的资源: {0}")]
    NotFound(String),
    
    /// 500 数据库错误，包含底层 sqlx 错误信息
    #[error("数据库操作失败")]
    Database(#[from] sqlx::Error),
    /// 500 服务端无法恢复的内部异常
    #[error("服务器内部错误")]
    Internal, 
}

/// 告诉 Axum 如何把 AppError 转换为一个完整 HTTP 响应
impl IntoResponse for AppError {
    /// 消费错误对象，并生成状态码与 JSON 响应体。
    fn into_response(self) -> axum::response::Response {
        // 根据具体错误类型计算 HTTP 状态码和对外错误消息。
        let status = match &self {
            // 400 请求参数错误
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            // 401 统一凭据错误、未授权访问
            AppError::InvalidCredentials | AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            // 404 资源未找到
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            
            // 500 数据库错误（包含底层 sqlx 错误信息）、服务端内部异常
            AppError::Database(_) | AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // 按项目统一协议构造错误响应 JSON
        let response = ApiResponse::<()> { code: status.as_u16(), msg: Cow::Owned(self.to_string()), data: None };

        // 将状态码和 JSON body 组合为 Axum 可发送的 HTTP 响应
        (status, Json(response)).into_response()
    }
}
