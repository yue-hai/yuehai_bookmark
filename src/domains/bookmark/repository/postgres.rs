//! 数据访问

// 导入 PostgreSQL 连接池类型，供仓储函数接收数据库访问能力
use sqlx::PgPool;
// 导入创建书签时需要写入数据库的请求 DTO
use crate::domains::bookmark::models::dto::create::CreateBookmarkReq;

/// 预留的书签插入函数，成功时返回新记录主键
pub async fn insert(_pool: &PgPool, _req: &CreateBookmarkReq) -> Result<i64, sqlx::Error> {
    // 数据库表和真实 SQL 尚未接入，当前固定返回模拟 ID 供 HTTP 流程联调。
    Ok(1)
}
