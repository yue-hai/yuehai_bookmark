//! 统一 JSON 提取器

use axum::{Json, extract::{FromRequest, Request, rejection::JsonRejection}, };
use serde::de::DeserializeOwned;
use crate::common::error::AppError;

/// 定义一个名为 AppJson 的元组结构体，包裹泛型 T
pub struct AppJson<T>(pub T);

/// 实现 FromRequest trait，使 AppJson 能够从 HTTP 请求中提取 JSON 数据
/// 
/// # 泛型约束：
/// * S: Send + Sync，表示状态类型必须是可发送和可同步的
/// * T: DeserializeOwned，表示数据类型必须是可反序列化的
impl<S, T> FromRequest<S> for AppJson<T> where S: Send + Sync, T: DeserializeOwned {
    // 指定如果提取失败，抛出 AppError
    type Rejection = AppError;
    
    /// 提取器的核心异步方法，从请求中提取 JSON 数据并进行错误处理
    /// 
    /// # Arguments
    /// * `request`：HTTP 请求对象
    /// * `state`：应用状态对象
    /// 
    /// # Returns
    /// * `Result<Self, Self::Rejection>`：成功时返回 AppJson 包裹的值，失败时返回 AppError
    async fn from_request(request: Request, state: &S, ) -> Result<Self, Self::Rejection> {
        // 使用 axum 的 Json 提取器从请求中提取 JSON 数据
        Json::<T>::from_request(request, state)
            // 等待异步解析完成
            .await
            // 如果解析成功，将提取到的 JSON 数据包裹在 AppJson 中
            .map(|Json(value)| AppJson(value))
            // 如果解析失败，将错误转换为自定义的 AppError
            .map_err(|rejection: JsonRejection| {
                // 根据不同的 JsonRejection 类型，返回对应的错误信息
                let message = match rejection {
                    JsonRejection::JsonDataError(_) => "请求字段格式不正确",
                    JsonRejection::JsonSyntaxError(_) => "请求体不是合法的 JSON",
                    JsonRejection::MissingJsonContentType(_) => "请求头必须包含 application/json",
                    JsonRejection::BytesRejection(_) => "读取请求体失败",
                    _ => "请求体格式不正确",
                };

                // 返回自定义的 BadRequest 错误
                AppError::BadRequest(message)
            })
    }
}