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

> 以下是项目的目标目录结构。目录按业务模块逐步创建，不为了“结构完整”而预先创建空文件。
> `migrations/` 中的文件是数据库升级时要执行的输入；`docs/` 中的文件仅用于说明和查阅。

```text
yuehai_bookmark/
├── Cargo.toml                            # 包元数据、Rust 版本和项目直接依赖
├── Cargo.lock                            # 应用依赖的精确锁定版本，必须提交到 Git
├── .env.example                          # 可提交的环境变量模板，不包含真实密码或 Token
├── .gitignore                            # 忽略 target、.env、IDE 本地配置等生成文件
├── migrations/                           # SQLx 按文件名顺序执行的数据库结构演进脚本
│   ├── 0001_create_users.sql             # 创建 users 表、约束、索引和更新时间触发器
│   ├── 0002_create_auth_sessions.sql     # 创建登录会话或 Refresh Token 的持久化表
│   ├── 0003_create_folders.sql           # 创建用户文件夹及其树形父子关系
│   ├── 0004_create_bookmarks.sql         # 创建书签、标签和关联关系
│   └── 0005_create_folder_shares.sql     # 创建文件夹分享、成员角色和邀请状态
├── scripts/                              # 本地开发或部署前执行的辅助脚本，不放业务代码
│   └── create_dev_database.ps1           # 创建本地开发数据库；不包含表结构迁移逻辑
├── docs/                                 # 面向开发者阅读的设计和使用文档，不作为运行时输入
│   ├── api.md                            # API 协议、示例请求、状态码和错误码约定
│   └── database.md                       # 表关系、索引策略和迁移说明
├── tests/                                # 黑盒集成测试：通过公开 Router 验证真实 HTTP 行为
│   ├── support/
│   │   └── mod.rs                        # 测试数据库、测试 AppState 和公共断言辅助函数
│   ├── auth_register.rs                  # 用户注册的成功、校验失败和邮箱冲突测试
│   └── bookmarks_create.rs               # 创建书签的认证、权限和写入测试
│
└── src/
    ├── main.rs                           # 二进制入口：加载配置、初始化应用并启动 HTTP 服务
    ├── lib.rs                            # 库入口：注册模块，并暴露供集成测试使用的应用构造函数
    │
    ├── app/                              # 应用装配：配置、全局状态、顶层路由和日志初始化
    │   ├── mod.rs                        # 聚合 app 子模块，避免 main.rs 了解具体实现细节
    │   ├── config.rs                     # 读取环境变量并解析为强类型 AppConfig，配置错误立即失败
    │   ├── state.rs                      # 定义 AppState，集中保存 PgPool 等跨请求共享资源
    │   ├── router.rs                     # 装配 API 版本前缀、中间件和所有业务模块路由
    │   └── telemetry.rs                  # 初始化 tracing 日志、日志级别和请求链路信息
    │
    ├── common/                           # 仅放跨业务模块、长期稳定复用的通用能力
    │   ├── mod.rs                        # 聚合错误和 HTTP 通用能力
    │   ├── error.rs                      # 定义 AppError 并将内部错误映射为安全的客户端错误
    │   └── http/                         # 与具体业务无关的 HTTP 协议实现
    │       ├── mod.rs                    # 聚合 HTTP 响应、提取失败和分页模块
    │       ├── response.rs               # 定义 Problem Details 等统一错误与成功响应结构
    │       ├── rejection.rs              # 将 Axum JSON、Query、Path 参数提取错误转换为统一响应
    │       └── pagination.rs             # 定义分页请求参数、分页结果和默认值校验
    │
    ├── infra/                            # 外部技术适配：不承载 auth、bookmarks 等业务规则
    │   ├── mod.rs                        # 聚合已经接入的外部技术能力
    │   ├── database/                     # PostgreSQL 与 SQLx 相关实现
    │   │   ├── mod.rs                    # 对外暴露数据库初始化和健康检查能力
    │   │   ├── pool.rs                   # 创建、配置和关闭 PostgreSQL 连接池
    │   │   ├── migration.rs              # 执行 SQLx 数据库迁移并报告失败原因
    │   │   └── health.rs                 # 执行轻量查询，供健康检查接口确认数据库可用
    │   └── security/                     # 密码和会话凭据等安全基础能力
    │       ├── mod.rs                    # 聚合密码与 Token 实现
    │       ├── password.rs               # 使用 Argon2id 生成和校验密码哈希，绝不记录明文密码
    │       └── token.rs                  # 生成、哈希和校验 Session Token、Refresh Token
    │
    └── modules/                          # 业务模块根目录；新增功能先新增模块，不修改既有模块边界
        ├── mod.rs                        # 注册 auth、bookmarks、folders、shares 等业务模块
        │
        ├── auth/                         # 认证模块：注册、登录、登出、会话续期和当前用户识别
        │   ├── mod.rs                    # 认证模块门面，对外暴露 router 等必要能力
        │   ├── routes.rs                 # 注册 /api/auth 下的认证相关路由
        │   ├── handlers/                 # HTTP 入口：提取请求、调用 service、转换为 HTTP 响应
        │   │   ├── mod.rs                # 聚合认证 Handler
        │   │   ├── register.rs           # 处理注册请求并在成功时返回 HTTP 201
        │   │   ├── login.rs              # 处理登录请求并返回受控的会话凭据
        │   │   ├── logout.rs             # 撤销当前会话或指定会话
        │   │   └── refresh.rs            # 校验 Refresh Token 并续期或轮换会话
        │   ├── dto/                      # HTTP 请求和响应结构；禁止包含 password_hash 等内部字段
        │   │   ├── mod.rs                # 聚合认证 DTO
        │   │   ├── register.rs           # RegisterRequest 和 RegisterResponse
        │   │   ├── login.rs              # LoginRequest 和 LoginResponse
        │   │   └── session.rs            # SessionResponse 和 RefreshRequest
        │   ├── service/                  # 认证业务流程：可调用 model、repository 和 infra
        │   │   ├── mod.rs                # 聚合注册、登录、登出和会话续期服务
        │   │   ├── register.rs           # 规范化邮箱、校验输入、哈希密码并创建用户
        │   │   ├── login.rs              # 验证密码、检查账号状态并创建认证会话
        │   │   ├── logout.rs             # 使指定会话立即失效
        │   │   └── session.rs            # 校验、轮换和延长 Refresh Token 对应的服务端会话
        │   ├── repository/               # 数据访问：只处理 SQL、行映射和数据库错误转换
        │   │   ├── mod.rs                # 聚合 users 和 sessions 仓储
        │   │   ├── users.rs              # 插入、按邮箱查询、读取和更新 users 表记录
        │   │   └── sessions.rs           # 创建、查询、续期和撤销 auth_sessions 表记录
        │   └── model/                    # 认证业务模型与校验规则；不依赖 Axum 或 SQLx
        │       ├── mod.rs                # 聚合 User、Email、Role、Status 等模型
        │       ├── user.rs               # User、UserId、DisplayName 等用户模型
        │       ├── email.rs              # Email 规范化、格式校验和安全显示规则
        │       ├── role.rs               # UserRole 与 AccountStatus 枚举
        │       └── error.rs              # EmailAlreadyRegistered 等认证业务错误
        │
        ├── bookmarks/                    # 书签模块：创建、查询、编辑、软删除和元数据刷新
        │   ├── mod.rs                    # 书签模块门面，对外暴露 router
        │   ├── routes.rs                 # 注册 /api/bookmarks 下的路由
        │   ├── handlers/
        │   │   ├── mod.rs                # 聚合书签 Handler
        │   │   ├── create.rs             # 创建书签 HTTP 入口
        │   │   ├── get.rs                # 按 ID 查询书签 HTTP 入口
        │   │   ├── list.rs               # 分页查询书签 HTTP 入口
        │   │   ├── update.rs             # 编辑书签 HTTP 入口
        │   │   └── delete.rs             # 软删除书签 HTTP 入口
        │   ├── dto/
        │   │   ├── mod.rs                # 聚合书签 DTO
        │   │   ├── create.rs             # CreateBookmarkRequest 和创建响应
        │   │   ├── update.rs             # UpdateBookmarkRequest 和更新响应
        │   │   └── view.rs               # BookmarkResponse 和分页列表响应
        │   ├── service/                  # 创建、查询、更新、删除和刷新书签的业务流程
        │   ├── repository/               # bookmarks、bookmark_tags 等表的 SQL 查询
        │   └── model/                    # Bookmark、BookmarkId、URL 校验和书签状态模型
        │
        ├── folders/                      # 文件夹模块：树形结构、移动、软删除和书签归档
        │   ├── mod.rs                    # 文件夹模块门面，对外暴露 router
        │   ├── routes.rs                 # 注册 /api/folders 下的路由
        │   ├── handlers/                 # 文件夹创建、移动、树查询、删除和恢复的 HTTP 入口
        │   ├── dto/                      # 文件夹请求和响应结构
        │   ├── service/                  # 文件夹树校验、移动、删除和恢复的业务流程
        │   ├── repository/               # folders 表和递归树查询 SQL
        │   └── model/                    # Folder、ParentFolderId 和循环引用校验规则
        │
        └── shares/                       # 分享模块：邀请、成员角色、接受、拒绝和撤销分享
            ├── mod.rs                    # 分享模块门面，对外暴露 router
            ├── routes.rs                 # 注册 /api/shares 下的路由
            ├── handlers/                 # 分享邀请、接受、拒绝和撤销的 HTTP 入口
            ├── dto/                      # 分享请求和响应结构
            ├── service/                  # 邀请成员、变更权限和撤销分享的业务流程
            ├── repository/               # folder_shares 表的 SQL 查询
            └── model/                    # ShareRole、InviteStatus 和访问权限规则

```

## 目录使用规则

1. `app/` 只处理启动、装配和全局配置。它通常只有 `config.rs`、`state.rs`、`router.rs`、`telemetry.rs` 等少量文件；没有出现多个同类文件前，不拆二级目录。
2. `common/` 只允许放至少被两个业务模块复用、且不包含业务名的代码。禁止创建 `utils.rs`、`helpers.rs`、`models.rs` 等无法表达职责的杂项目录或文件。
3. `infra/` 按外部技术而非业务拆分。数据库、密码、缓存、邮件、对象存储分别位于独立目录；未接入的技术不预先创建空目录。
4. `modules/` 按业务能力拆分。业务模块内固定使用 `routes`、`handlers`、`dto`、`service`、`repository`、`model`；代码增长时仅在模块内部继续拆文件。
5. 当同一职责出现三个以上相关文件，或需要独立初始化、测试、错误处理时，才创建子目录。否则保留单文件，避免无意义的 `model` 和空目录。

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

