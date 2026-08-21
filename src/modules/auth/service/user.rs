//! 用户相关 service

use crate::app::state::AppState;
use crate::common::error::AppError;
use crate::modules::auth::dto::login::LoginResponse;
use crate::modules::auth::dto::register::RegisterRequest;
use crate::modules::auth::repository::{auth, user};
use crate::modules::auth::service::auth::issue_session_token;
use crate::modules::auth::service::password;

/// 注册 service，处理注册请求的业务逻辑，包括验证用户信息、创建新用户并返回给客户端
/// 
/// # Arguments
/// * `state`：应用状态，包含数据库连接池等共享资源
/// * `request`：注册请求，包含邮箱、密码和显示名称
/// 
/// # Returns
/// * `Result<LoginResponse, AppError>`：成功时返回登录响应，包含用户信息，失败时返回应用错误
pub async fn register(state: &AppState, request: RegisterRequest) -> Result<LoginResponse, AppError> {
    // 验证注册请求，确保参数符合要求
    let request = request.validate()?;

    // 在 Tokio 专用阻塞线程池中使用 Argon2 哈希用户密码，避免占用异步工作线程
    let password_hash = password::hash_password(request.password).await?;

    // 从连接池取得一条连接，并开始数据库事务
    let mut transaction = state.database_pool.begin().await?;
    // 尝试异步将新注册的用户信息插入数据库
    let new_user = match user::insert_user(&mut transaction, &request.email, &password_hash, &request.display_name).await {
        // 插入成功，提取出构建好的新 User 实体
        Ok(user) => user,

        // 边界情况：用户重复，23505 是 PostgreSQL 的标准错误码，代表违反了 Unique 约束
        Err(sqlx::Error::Database(error))
        if error.code().as_deref() == Some("23505") => {
            return Err(AppError::BadRequest("邮箱已经注册"));
        }

        // 异常情况：其他未知的数据库报错，直接向上层抛出
        Err(error) => {
            return Err(AppError::Database(error));
        }
    };

    // 签发 Token，实现注册即登录
    let (access_token, token_hash) = issue_session_token(state.token_hash_secret.as_bytes())?;
    // 持久化当前用户的 Session Token 哈希
    auth::insert_session(&mut transaction, new_user.id, &token_hash, state.token_expire_days).await?;
    // 只有 users 与 auth_sessions 都成功写入时才提交事务，否则在函数返回时自动回滚，保证数据一致性
    transaction.commit().await?;

    // 构造并返回登录成功响应
    Ok(LoginResponse::from_user(new_user, access_token, "Bearer"))
}
