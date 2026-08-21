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

> 以下目录树以当前代码为准。`bookmarks/`、`folders/` 保留为后续实现时的参考骨架；其中尚未接入路由或业务逻辑的文件不代表功能已经实现。
>
> 当前数据库初始化脚本是 `docs/database/init.sql`，它用于首次创建开发数据库；项目目前**尚未**使用 SQLx migrations。未来需要演进已有数据库结构时，再新增 `migrations/` 目录并启用迁移功能。

```text
yuehai_bookmark/
├── Cargo.toml                            # 包元数据、Rust 版本、直接依赖及其 features
├── Cargo.lock                            # 应用实际使用的精确依赖版本，应提交到 Git
├── .env                                  # 本地环境变量：数据库连接和 Token HMAC 密钥，不提交到 Git
├── .gitignore                            # 忽略 target、.env 与 IDE 本地配置等生成文件
├── README.md                             # 项目说明、当前结构和命令记录
├── docs/
│   └── database/
│       └── init.sql                      # 当前开发数据库的初始化脚本：表、索引、约束和触发器
│
└── src/
    ├── main.rs                           # 二进制入口：读取配置、创建连接池并启动 Axum 服务
    ├── lib.rs                            # 库入口：导出 app、common、infra 和 modules 模块树
    │
    ├── app/                              # 应用装配：配置、共享状态和顶层路由
    │   ├── mod.rs                        # 聚合 app 子模块
    │   ├── config.rs                     # 读取 .env / 环境变量并构造 AppConfig
    │   ├── state.rs                      # AppState：PgPool、Token 过期天数和 HMAC 密钥
    │   └── router.rs                     # 装配 /api 前缀并合并各业务模块路由
    │
    ├── common/                           # 跨业务模块复用的通用能力
    │   ├── mod.rs                        # 聚合错误与 HTTP 通用模块
    │   ├── error.rs                      # AppError 及其到统一 HTTP 错误响应的转换
    │   └── http/
    │       ├── mod.rs                    # 聚合通用 HTTP 类型
    │       ├── json.rs                   # AppJson<T>：统一 JSON 请求体解析失败的响应格式
    │       └── response.rs               # ApiResponse<T>：统一成功响应格式
    │
    ├── infra/                            # 外部技术适配层，不承载具体业务规则
    │   ├── mod.rs                        # 聚合基础设施模块
    │   └── database/
    │       ├── mod.rs                    # 聚合数据库模块
    │       └── pool.rs                   # 创建 PostgreSQL SQLx 连接池
    │
    └── modules/                          # 按业务领域组织的模块
        ├── mod.rs                        # 注册 auth 和 bookmarks 模块
        │
        ├── auth/                         # 已实现：注册、登录、Session Token 验证和当前用户查询
        │   ├── mod.rs                    # 聚合认证子模块
        │   ├── routes.rs                 # 注册 /api/auth/register、/login、/me 路由
        │   ├── dto/                      # HTTP 请求与响应 DTO；不暴露 password_hash
        │   │   ├── mod.rs                # 聚合认证 DTO
        │   │   ├── register.rs           # RegisterRequest：校验并规范化注册输入
        │   │   ├── login.rs              # LoginRequest、LoginResponse
        │   │   └── me.rs                 # MeResponse：当前用户公开信息
        │   ├── extractors/               # 可复用的 HTTP 请求提取器
        │   │   ├── mod.rs                # 聚合认证提取器
        │   │   └── current_user.rs       # CurrentUser：读取 Bearer Token 并完成认证
        │   ├── handlers/                 # HTTP 入口：提取请求、调用 service、返回 ApiResponse
        │   │   ├── mod.rs                # 聚合认证 Handler
        │   │   ├── auth.rs               # POST /login 与 GET /me
        │   │   └── user.rs               # POST /register
        │   ├── service/                  # 认证业务流程
        │   │   ├── mod.rs                # 聚合认证 service
        │   │   ├── auth.rs               # 登录、HMAC Token 签发与 Token 认证
        │   │   └── user.rs               # 注册：Argon2 密码哈希及创建用户、Session 的事务
        │   ├── repository/               # 数据访问：SQL 与数据库行映射
        │   │   ├── mod.rs                # 聚合认证 repository
        │   │   ├── user.rs               # 插入用户、按邮箱或 ID 查询有效用户
        │   │   └── auth.rs               # 创建并查询未过期、未撤销的 Session
        │   └── model/                    # 内部数据库模型
        │       ├── mod.rs                # 聚合认证模型
        │       ├── user.rs               # User、SystemRole、UserStatus
        │       └── auth_session.rs       # AuthSession
        │
        └── bookmarks/                    # 当前仅为后续书签功能保留的参考骨架
            ├── mod.rs                    # 书签模块入口
            ├── dto/                      # 未来放置创建、更新和列表请求/响应 DTO
            ├── handlers/                 # 未来放置 HTTP Handler
            ├── model/                    # 未来放置 Bookmark 等内部模型
            ├── repository/               # 未来放置 bookmarks 表相关 SQL
            └── service/                  # 未来放置书签业务规则
```

认证模块当前调用方向：

```text
routes → handlers → service → repository → PostgreSQL
                 ↘ extractors → service
```

- `handlers` 不直接编写 SQL；
- `repository` 不负责 HTTP 请求校验；
- `CurrentUser` 负责通用 Bearer Token 验证，受保护接口可直接提取已认证用户；
- 用户密码使用 Argon2 与随机盐；Session Token 使用 HMAC-SHA-256，数据库只保存其 HMAC 结果。

# 五、核心依赖包 (Crates)

以下版本与当前 `Cargo.toml` 保持一致；Cargo.lock 会锁定本次实际解析出的精确版本。

## 1、tokio

1. 当前声明版本：`1.53.1`
2. 地址：https://crates.io/crates/tokio
3. 用途：Rust 异步运行时；`#[tokio::main]`、`TcpListener`、Axum 服务运行在 Tokio 上。
4. 当前 features：`macros`、`rt-multi-thread`、`net`。

## 2、axum

1. 当前声明版本：`0.8.9`
2. 地址：https://crates.io/crates/axum
3. 用途：HTTP 路由、Handler、`State`、`Json`、请求提取器与响应转换。
4. 当前 feature：`json`，用于 JSON 请求和响应支持。

## 3、serde

1. 当前声明版本：`1.0.229`
2. 地址：https://crates.io/crates/serde
3. 用途：将 JSON 请求体反序列化为 DTO，并将响应 DTO 序列化为 JSON。
4. 当前 feature：`derive`，用于 `#[derive(Deserialize, Serialize)]`。
5. 项目没有直接依赖 `serde_json`；JSON 编解码由 Axum 的 `Json` 提取器和响应类型间接完成。

## 4、sqlx

1. 当前声明版本：`0.9.0`
2. 地址：https://crates.io/crates/sqlx
3. 用途：PostgreSQL 异步连接池、事务、SQL 查询与查询结果映射。
4. 当前 features：
   - `postgres`：PostgreSQL 支持；
   - `runtime-tokio`：在 Tokio 运行时中执行异步数据库操作；
   - `macros`：使用 `FromRow`、`Type` 等派生宏；
   - `chrono`：映射 PostgreSQL 时间类型与 `chrono::DateTime<Utc>`；
   - `ipnetwork`：映射 PostgreSQL `INET` 字段与 `std::net::IpAddr`。
5. 当前使用运行时查询 API（如 `query_as`）；SQL 不会在编译时连接数据库校验。

## 5、chrono

1. 当前声明版本：`0.4`
2. 地址：https://crates.io/crates/chrono
3. 用途：表示并序列化用户、Session 的 `TIMESTAMPTZ` 时间字段，例如 `created_at`、`expires_at`。
4. 当前 feature：`serde`，用于将时间字段输出为 JSON。

## 6、dotenvy

1. 当前声明版本：`0.15`
2. 地址：https://crates.io/crates/dotenvy
3. 用途：本地开发时读取项目根目录 `.env`；部署环境仍应由系统或平台注入环境变量。

## 7、thiserror

1. 当前声明版本：`2.0.19`
2. 地址：https://crates.io/crates/thiserror
3. 用途：通过 `#[derive(Error)]` 实现 `AppError` 的 `Display`、`Error` 与错误来源转换，减少手写样板代码。

## 8、argon2

1. 当前声明版本：`0.5`
2. 地址：https://crates.io/crates/argon2
3. 用途：使用 Argon2id 哈希用户密码，并在登录时验证密码。
4. 密码哈希使用每个密码独立的随机盐；不要将 Argon2 用于可索引的 Session Token 查询。

## 9、getrandom

1. 当前声明版本：`0.4.3`
2. 地址：https://crates.io/crates/getrandom
3. 用途：从操作系统安全随机源生成密码盐与 32 字节 Session Token。

## 10、hmac 与 sha2

1. 当前声明版本：`hmac 0.13`、`sha2 0.11`
2. 地址：https://crates.io/crates/hmac 、https://crates.io/crates/sha2
3. 用途：使用 `HMAC-SHA-256` 计算 Session Token 的确定性摘要。
4. `TOKEN_HASH_SECRET` 仅保存于服务端；原始 Token 只返回客户端，数据库只保存固定 64 字符的 HMAC 结果，以便通过唯一索引直接查询。

# 六、核心 API 预览
