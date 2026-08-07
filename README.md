# 一、各种版本

1. Rust 工具链 (rustc / cargo)：1.97.1 (stable-x86_64-pc-windows-msvc)
2. IDE：IntelliJ IDEA Ultimate (2026.1.3) + Official Rust Plugin
3. 数据库：postgres 17
4. 包管理工具：Cargo

# 二、运行环境

1. 开发环境：Windows 10/11 (依赖 MSVC v143 C++ Build Tools)
2. 生产部署：Linux (Ubuntu 26.04 / Alpine) / Docker 容器化部署
3. 架构支持：x86_64, aarch64 (支持跨平台编译)

# 三、命令记录

## 1、开发与调试命令

```shell
# 快速检查代码是否能通过编译（不生成二进制文件，速度极快，开发最常用）
cargo check

# 格式化所有代码（遵循官方规范）
cargo fmt

# 运行静态分析与代码规范检查 (Linter)
cargo clippy -- -D warnings

# 本地运行开发服务
cargo run

```

## 2、打包与部署命令

```shell
# 清理编译缓存（类似 flutter clean / mvn clean）
cargo clean

# 构建生产环境 Release 包（极致优化，编译较慢但运行极快）
cargo build --release

# 运行生产环境产物（Linux/macOS）
./target/release/yuehai_bookmark

# 运行生产环境产物（Windows）
.\target\release\yuehai_bookmark.exe

```

# 四、项目结构

```text
yuehai_bookmark/
├── Cargo.toml                    # 依赖清单
├── Cargo.lock                    # 依赖版本锁
├── .env                          # 本地环境变量配置 (数据库密码等，不提交 Git)
│
└── src/
    ├── main.rs                   # 纯粹的启动器：负责初始化环境并挂载端口
    ├── lib.rs                    # 根模块树注册表：编织所有目录，方便集成测试
    │
    ├── config/                   # 【配置模块】
    │   ├── mod.rs
    │   └── env.rs                # 负责读取和强类型校验 .env 配置
    │
    ├── core/                     # 【全局核心基建】(与具体业务无关的代码)
    │   ├── mod.rs
    │   ├── error.rs              # 核心：定义全局统一的 AppError 枚举
    │   ├── state.rs              # 核心：跨 Handler 传递的应用状态 (DB Pool)
    │   └── response.rs           # 统一 JSON 返回体包装 (如 code, msg, data)
    │
    ├── infrastructure/           # 【基础设施层】
    │   ├── mod.rs
    │   └── database.rs           # 负责初始化 postgres 连接池
    │
    └── domains/                  # 【业务领域聚合】(极其细化)
        └── bookmark/             # 书签领域闭环
            ├── mod.rs            # 领域门面：拼装路由并对外暴露 bookmark::router()
            │
            ├── models/           # 1. 模型定义
            │   ├── mod.rs
            │   ├── entity.rs     # 数据库底层表结构映射
            │   └── dto/          # Data Transfer Objects
            │       ├── mod.rs
            │       ├── create.rs # POST 创建接口的 Request 结构体
            │       └── view.rs   # 响应给前端的视图结构体
            │
            ├── handlers/         # 2. 控制器 (HTTP 逻辑隔离)
            │   ├── mod.rs        
            │   ├── create.rs     # 新增书签逻辑
            │   ├── get.rs        # 按 ID 查询单条记录
            │   ├── list.rs       # 分页列表查询
            │   ├── update.rs     # 更新逻辑
            │   └── delete.rs     # 删除逻辑
            │
            └── repository/       # 3. 数据持久化
                ├── mod.rs
                └── postgres.rs     # 书签表相关的底层 SQL 操作

```

# 五、核心依赖包 (Crates)

## 1、tokio

1. 版本：1.0+
2. 地址：https://crates.io/crates/tokio
3. 描述：
   1. Rust 事实上的异步标准运行时（Runtime）
   2. 提供事件循环、异步 I/O、定时器等底层多线程并发支持



## 2、axum

1. 版本：0.7+
2. 地址：https://crates.io/crates/axum
3. 描述：
   1. 由 Tokio 官方团队维护的极其优雅、高性能的 Web 框架
   2. 提供类型安全的路由机制和基于宏的无开销请求参数提取器 (Extractor)



## 3、serde & serde_json

1. 版本：1.0+
2. 地址：https://crates.io/crates/serde
3. 描述：
   1. Rust 生态中最强大的序列化/反序列化框架
   2. 依赖编译期宏展开，实现零运行时开销的 JSON 与结构体互转


## 4、dotenvy

1. 版本：0.15+
2. 地址：https://crates.io/crates/dotenvy
3. 描述：
   1. 用于处理环境变量的库，支持从 .env 文件中加载配置，并提供类型安全的访问方式
   2. 支持在不同环境下的配置管理，方便开发和部署


## 5、sqlx

1. 版本：0.8+
2. 地址：https://crates.io/crates/sqlx
3. 描述：
   1. 纯异步、100% 纯 Rust 编写的数据库驱动
   2. 支持编译期 SQL 语句合法性检查（如果不写对 SQL，代码直接无法编译），支持 SQLite/PostgreSQL/MySQL



# 六、核心 API 预览

