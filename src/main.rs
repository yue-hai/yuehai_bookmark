//! HTTP 服务启动入口

// 导入 Axum 的路由构造器
use axum::Router;
// 导入当前项目的配置、共享状态、业务路由和基础设施模块。
use yuehai_bookmark::{config, core::state::AppState, domains, infrastructure};

/// 声明异步程序入口，并在底层自动创建并启动 Tokio 多线程运行时 <br>
/// 因为 Rust 标准库本身不自带异步运行时，原生的 main 函数不允许是 async 的 <br>
/// 该宏会在编译期拦截 `async fn main()` 代码块，将其重写为标准的同步 `main` 函数 <br>
/// 并在内部初始化事件循环（Event Loop）和工作线程池，最后阻塞主线程直到所有异步任务执行完毕 <br>
#[tokio::main]
async fn main() {
    // 1、尝试加载本地 .env，初始化环境变量；未找到 .env 文件时不会报错，继续使用系统环境变量
    config::env::init();
    
    // 2、获取环境变量配置
    let (state, bind_addr) = {
        // 服务器监听地址
        let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        // 服务器监听端口
        let server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
        // 数据库连接字符串，必须设置
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL 必须设置");
        
        // 3、异步建立 PostgreSQL 连接池
        let pg_pool = infrastructure::database::init_pool(&db_url).await;
        
        // 返回元组：(全局状态, 拼接好的监听地址)
        (AppState { db_pool: pg_pool }, format!("{}:{}", server_host, server_port))
    };

    // 4、创建顶层 Router 并装配书签领域子路由
    let app = Router::new()
        // 将书签领域路由嵌套到实际访问前缀 `/bookmarks`
        .nest("/bookmarks", domains::bookmark::router())
        // 向 Router 注入状态，Handler 可通过 State<AppState> 自动提取
        .with_state(state);

    // 5、异步绑定 TCP 端口；端口被占用时直接暴露启动错误。
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    println!("🚀 服务启动成功，监听地址: http://{}", listener.local_addr().unwrap());
    
    // 6、将监听器和 Router 交给 Axum，持续处理 HTTP 请求
    axum::serve(listener, app).await.unwrap();
}
