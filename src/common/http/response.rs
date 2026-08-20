//! 定义 API 的统一成功响应格式
//!
//! 所有成功接口使用 `code`、`msg`、`data` 三个字段的统一格式
//! `data` 为 `Option<T>`，支持没有业务数据的成功结果

use std::borrow::Cow;
use axum::{Json, response::IntoResponse};
use serde::Serialize;

/// 泛型 API 成功响应体，其中 T 是具体业务数据的类型
#[derive(Serialize)]
pub struct ApiResponse<T> {
    /// 业务状态码
    pub code: u16,
    /// 供客户端展示或调试的响应说明
    pub msg: Cow<'static, str>,
    /// 可选的业务数据；Some 表示有数据，None 表示无数据
    pub data: Option<T>,
}

/// 仅当业务数据本身可序列化时，才允许构造 API 响应
impl<T: Serialize> ApiResponse<T> {
    /// 标准成功 JSON 响应
    pub fn success(data: T) -> impl IntoResponse {
        // 通过 Json 包装器让 Axum 自动设置 JSON 响应头并完成序列化
        Json(ApiResponse { code: 200, msg: Cow::Borrowed("Success"), data: Some(data) })
    }

    /// 标准成功 JSON 响应
    pub fn success_with_msg(data: T, msg: String) -> impl IntoResponse {
        Json(ApiResponse { code: 200, msg: Cow::Owned(msg), data: Some(data) })
    }
}
