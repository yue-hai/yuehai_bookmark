//! 定义 API 的统一成功响应格式
//!
//! 所有成功接口使用 `code`、`msg`、`data` 三个字段的统一格式
//! `data` 为 `Option<T>`，支持没有业务数据的成功结果

// 导入 Rust 标准库的 Cow 类型，用于在响应消息中高效地处理静态字符串和动态字符串
use std::borrow::Cow;
// 导入 Axum 的响应 trait 与 JSON 包装器。
use axum::{Json, response::IntoResponse};
// 导入 Serialize 派生宏，使结构体能够编码为 JSON。
use serde::Serialize;

/// 泛型 API 成功响应体，其中 T 是具体业务数据的类型
/// 宏 `#[derive(Serialize)]` 使得 Axum 能够配合 serde_json 自动完成序列化
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
    /// 标准成功 JSON 响应：可以直接返回业务数据，使用默认成功消息
    pub fn success(data: T) -> impl IntoResponse {
        // 通过 Json 包装器让 Axum 自动设置 JSON 响应头并完成序列化
        Json(ApiResponse {
            // 业务状态码
            code: 200,
            // 供客户端展示或调试的响应说明
            msg: Cow::Borrowed("Success"),
            // 将传入的业务数据放入 Some，表示响应包含有效数据
            data: Some(data),
        })
    }

    /// 标准成功 JSON 响应：可以直接返回业务数据，并允许自定义成功消息
    pub fn success_with_msg(data: T, msg: String) -> impl IntoResponse {
        Json(ApiResponse {
            // 业务状态码
            code: 200,
            // 使用 Cow::Owned 将动态字符串 msg 包装为可序列化的响应消息
            msg: Cow::Owned(msg),
            // 将传入的业务数据放入 Some，表示响应包含有效数据
            data: Some(data),
        })
    }
}
