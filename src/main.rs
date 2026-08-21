//! HTTP 服务启动入口

use std::error::Error;
use yuehai_bookmark::app::state::AppState;
use yuehai_bookmark::app::{config, router};
use yuehai_bookmark::infra::database::pool;

/// 声明异步程序入口，并在底层自动创建并启动 Tokio 多线程运行时 <br>
/// 因为 Rust 标准库本身不自带异步运行时，原生的 main 函数不允许是 async 的 <br>
/// 该宏会在编译期拦截 `async fn main()` 代码块，将其重写为标准的同步 `main` 函数 <br>
/// 并在内部初始化事件循环（Event Loop）和工作线程池，最后阻塞主线程直到所有异步任务执行完毕 <br>
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 加载 .env，并读取所有应用配置
    let config = config::from_env();

    // 创建 PostgreSQL 连接池
    let database_pool = pool::init_pool(&config.database_url).await?;
    // 创建 Axum 请求共享状态
    let state = AppState { database_pool, token_expire_days: config.token_expire_days };
    // 创建包含所有业务模块路由的应用 Router
    let app = router::build(state);
    
    // 异步绑定监听地址、TCP 端口
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await.expect("无法绑定 HTTP 服务监听地址");
    println!("🚀 服务启动成功，监听地址: http://{}", listener.local_addr()?);
    
    // 将监听器和 Router 交给 Axum，持续处理 HTTP 请求
    axum::serve(listener, app).await?;

    // 程序正常结束返回 Ok(())
    Ok(())
}
