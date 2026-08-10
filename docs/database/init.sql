-- =============================================================================
-- Yuehai Bookmark：数据库初始化脚本
-- =============================================================================
-- 目标版本：PostgreSQL 17
-- 执行客户端：psql
-- 数据库名称：yuehai_bookmark
-- 包含对象：数据库、函数、数据表、约束、索引、触发器和注释
-- 执行命令：psql -v ON_ERROR_STOP=1 -f database/init.sql postgres
-- ON_ERROR_STOP 会在任一语句失败时立即终止执行

-- 判断数据库是否存在，不存在时返回创建数据库的 SQL 语句，并使用 \gexec 执行
SELECT 'CREATE DATABASE yuehai_bookmark WITH ENCODING ''UTF8'' TEMPLATE template0'
WHERE NOT EXISTS (
    SELECT 1
    FROM pg_database
    WHERE datname = 'yuehai_bookmark'
) \gexec

-- 切换数据库至 yuehai_bookmark，后续所有对象都在该数据库中创建
\connect yuehai_bookmark
-- 开始事务，确保在执行过程中出现错误时可以回滚
BEGIN;

-- =============================================================================
-- 0. 公共函数
-- =============================================================================

-- 创建统一更新时间函数，供包含 updated_at 的表复用
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    -- 每次更新记录时，自动刷新 updated_at
    NEW.updated_at = CURRENT_TIMESTAMP;
    -- 返回修改后的新记录，让原 UPDATE 继续执行
    RETURN NEW;
END;
$$;

-- =============================================================================
-- 1. users：用户
-- =============================================================================

-- 登录账号、密码哈希、系统角色和账号状态
CREATE TABLE users (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- id：用户主键；由 PostgreSQL 自动生成
    email               VARCHAR(320) NOT NULL,                          -- email：登录邮箱；去除首尾空格后长度为 3-320；Rust 存储前转小写；未删除账号唯一
    password_hash       VARCHAR(255) NOT NULL,                          -- password_hash：密码哈希；不能为空；保存 Argon2id 哈希，禁止明文
    display_name        VARCHAR(100) NOT NULL,                          -- display_name：显示名称；去除首尾空格后长度为 1-100；Rust 存储前规范化；不用于登录
    system_role         VARCHAR(20) NOT NULL DEFAULT 'user',            -- system_role：系统角色；默认 user；仅 user 或 admin
    status              VARCHAR(20) NOT NULL DEFAULT 'active',          -- status：账号状态；默认 active；仅 pending、active 或 disabled
    email_verified_at   TIMESTAMPTZ,                                    -- email_verified_at：邮箱验证时间；为空表示尚未验证
    last_login_at       TIMESTAMPTZ,                                    -- last_login_at：最近登录时间；记录最近一次成功登录
    created_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- created_at：创建时间；记录创建时刻
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- updated_at：更新时间；由触发器自动维护
    deleted_at          TIMESTAMPTZ,                                    -- deleted_at：软删除时间；为空表示账号未删除

    -- email：去除首尾空格后的长度必须为 3-320
    CONSTRAINT ck_users_email_length CHECK (char_length(btrim(email)) BETWEEN 3 AND 320),
    -- password_hash：禁止空字符串
    CONSTRAINT ck_users_password_hash_not_blank CHECK (char_length(password_hash) > 0),
    -- display_name：去除首尾空格后的长度必须为 1-100
    CONSTRAINT ck_users_display_name_length CHECK (char_length(btrim(display_name)) BETWEEN 1 AND 100),
    -- system_role：仅支持普通用户和管理员
    CONSTRAINT ck_users_system_role CHECK (system_role IN ('user', 'admin')),
    -- status：禁止写入未定义账号状态
    CONSTRAINT ck_users_status CHECK (status IN ('pending', 'active', 'disabled'))
);

-- 保证未删除用户的邮箱去除首尾空格并忽略大小写后唯一
CREATE UNIQUE INDEX uq_users_email_lower ON users (lower(btrim(email))) WHERE deleted_at IS NULL;
-- 加速按账号状态查询未删除用户
CREATE INDEX ix_users_status ON users (status) WHERE deleted_at IS NULL;
-- 在更新用户时自动维护 updated_at
CREATE TRIGGER trg_users_set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
-- 将用户表说明写入 PostgreSQL 元数据
COMMENT ON TABLE users IS '系统用户账号；密码只保存哈希，系统权限暂时使用 system_role';


-- =============================================================================
-- 2. auth_sessions：登录会话
-- =============================================================================

-- 保存服务端 Session 或 Refresh Token 的哈希
CREATE TABLE auth_sessions (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- id：会话主键；由 PostgreSQL 自动生成
    user_id         BIGINT NOT NULL,                                 -- user_id：所属用户；用户删除时级联删除其全部会话
    token_hash      VARCHAR(128) NOT NULL,                           -- token_hash：不能为空且全局唯一；禁止保存原始 Token
    expires_at      TIMESTAMPTZ NOT NULL,                            -- expires_at：过期时间；由 Rust 认证服务校验
    last_used_at    TIMESTAMPTZ,                                     -- last_used_at：最近使用时间；记录最后一次有效访问
    revoked_at      TIMESTAMPTZ,                                     -- revoked_at：撤销时间；为空表示会话未撤销
    ip_address      INET,                                            -- ip_address：客户端 IP；创建或最近使用会话的来源地址
    user_agent      TEXT,                                            -- user_agent：客户端标识；用于设备识别
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- created_at：创建时间；记录会话签发时刻

    -- user_id：必须引用有效用户；用户删除时级联清理会话
    CONSTRAINT fk_auth_sessions_user FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    -- token_hash：必须全局唯一
    CONSTRAINT uq_auth_sessions_token_hash UNIQUE (token_hash),
    -- token_hash：禁止空字符串
    CONSTRAINT ck_auth_sessions_token_hash_not_blank CHECK (char_length(token_hash) > 0)
);

-- 加速查询用户当前未撤销的会话
CREATE INDEX ix_auth_sessions_active_user
    ON auth_sessions (user_id, expires_at DESC)
    WHERE revoked_at IS NULL;
-- 将会话表说明写入 PostgreSQL 元数据
COMMENT ON TABLE auth_sessions IS '登录会话或 Refresh Token；只保存 Token 哈希';


-- =============================================================================
-- 3. folders：文件夹
-- =============================================================================

-- 保存用户文件夹，通过 parent_id 形成可嵌套树结构
CREATE TABLE folders (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- id：文件夹主键；非空且由 PostgreSQL 自动生成
    owner_id        BIGINT NOT NULL,                                 -- owner_id：必填；关联真实所有者，存在文件夹时禁止删除所有者
    parent_id       BIGINT,                                          -- parent_id：可空；父文件夹必须属于同一所有者且不能指向自身，多层循环由 Rust Service 校验
    name            VARCHAR(200) NOT NULL,                           -- name：必填；去除首尾空格后长度为 1 到 200；Rust 后台负责规范化，同级未删除目录中名称唯一
    description     TEXT,                                            -- description：文件夹备注
    sort_order      BIGINT NOT NULL DEFAULT 1024,                    -- sort_order：同一 owner_id、parent_id 层级内的排序值，推荐使用 1024 间隔排序
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- created_at：文件夹创建时间
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- updated_at：文件夹最后更新时间
    deleted_at      TIMESTAMPTZ,                                     -- deleted_at：软删除时间；为空表示正常

    -- owner_id：必须引用有效用户
    CONSTRAINT fk_folders_owner FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE RESTRICT,
    -- owner_id、parent_id：父子文件夹必须属于同一所有者
    CONSTRAINT fk_folders_parent_same_owner
        FOREIGN KEY (owner_id, parent_id)
        REFERENCES folders (owner_id, id)
        ON DELETE RESTRICT,
    -- owner_id、id：为同所有者资源的复合外键提供唯一候选键
    CONSTRAINT uq_folders_owner_id UNIQUE (owner_id, id),
    -- 文件夹名称去除首尾空格后必须仍有内容，规范化由 Rust 后台负责
    CONSTRAINT ck_folders_name CHECK (char_length(btrim(name)) BETWEEN 1 AND 200),
    -- 文件夹不能直接把自己设为父节点；多层循环由 Service 检查
    CONSTRAINT ck_folders_not_self_parent CHECK (parent_id IS NULL OR parent_id <> id)
);

-- 保证同一父节点下未删除文件夹的规范化名称忽略大小写后唯一；0 表示根层级
CREATE UNIQUE INDEX uq_folders_active_sibling_name
    ON folders (owner_id, COALESCE(parent_id, 0), lower(btrim(name)))
    WHERE deleted_at IS NULL;

-- 加速按所有者、父节点和排序值读取文件夹树
CREATE INDEX ix_folders_owner_tree
    ON folders (owner_id, parent_id, sort_order, id)
    WHERE deleted_at IS NULL;

-- 加速查询用户回收站中的文件夹
CREATE INDEX ix_folders_deleted
    ON folders (owner_id, deleted_at)
    WHERE deleted_at IS NOT NULL;

-- 在更新文件夹时自动维护 updated_at
CREATE TRIGGER trg_folders_set_updated_at
BEFORE UPDATE ON folders
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- 将文件夹表说明写入 PostgreSQL 元数据
COMMENT ON TABLE folders IS '用户拥有的嵌套文件夹；共享时只创建访问引用';


-- =============================================================================
-- 4. bookmarks：书签
-- =============================================================================

-- 保存书签主体、用户笔记和排序信息
CREATE TABLE bookmarks (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- id：书签主键；由 PostgreSQL 自动生成
    owner_id        BIGINT NOT NULL,                                 -- owner_id：真实所有者；存在书签时禁止删除该用户
    folder_id       BIGINT,                                          -- folder_id：所属文件夹；为空表示未分类；非空时必须属于同一所有者
    title           VARCHAR(500) NOT NULL,                           -- title：书签标题；去除首尾空格后长度为 1-500；网页抓取不会覆盖此字段
    url             TEXT NOT NULL,                                   -- url：原始 URL；去除首尾空格后长度为 1-8192；Rust 负责规范化和协议校验；允许重复
    normalized_url  TEXT,                                            -- normalized_url：规范化 URL；用于搜索和重复提示，不强制唯一
    description     TEXT,                                            -- description：书签描述；用户填写的简短说明
    sort_order      BIGINT NOT NULL DEFAULT 1024,                    -- sort_order：在所属文件夹内排序
    is_favorite     BOOLEAN NOT NULL DEFAULT FALSE,                  -- is_favorite：收藏标记；true 表示已收藏
    is_archived     BOOLEAN NOT NULL DEFAULT FALSE,                  -- is_archived：归档标记；true 表示已归档
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- created_at：创建时间；记录书签创建时刻
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- updated_at：更新时间；由触发器自动维护
    deleted_at      TIMESTAMPTZ,                                     -- deleted_at：软删除时间；为空表示书签未删除

    -- owner_id：必须引用有效用户
    CONSTRAINT fk_bookmarks_owner FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE RESTRICT,
    -- owner_id、folder_id：书签只能关联同一所有者的文件夹
    CONSTRAINT fk_bookmarks_folder_same_owner
        FOREIGN KEY (owner_id, folder_id)
        REFERENCES folders (owner_id, id)
        ON DELETE RESTRICT,
    -- title：去除首尾空格后的长度必须为 1-500
    CONSTRAINT ck_bookmarks_title CHECK (char_length(btrim(title)) BETWEEN 1 AND 500),
    -- url：去除首尾空格后不能为空，且长度不得超过 8192
    CONSTRAINT ck_bookmarks_url CHECK (char_length(btrim(url)) BETWEEN 1 AND 8192)
);

-- 加速读取某个文件夹中的未删除书签，并保持稳定排序
CREATE INDEX ix_bookmarks_owner_folder_order
    ON bookmarks (owner_id, folder_id, sort_order, id)
    WHERE deleted_at IS NULL;

-- 加速按创建时间倒序查询用户书签
CREATE INDEX ix_bookmarks_owner_created
    ON bookmarks (owner_id, created_at DESC, id DESC)
    WHERE deleted_at IS NULL;

-- 加速查询收藏书签
CREATE INDEX ix_bookmarks_owner_favorite
    ON bookmarks (owner_id, updated_at DESC)
    WHERE deleted_at IS NULL AND is_favorite = TRUE;

-- 加速查询归档书签
CREATE INDEX ix_bookmarks_owner_archived
    ON bookmarks (owner_id, updated_at DESC)
    WHERE deleted_at IS NULL AND is_archived = TRUE;

-- 加速查询回收站中的书签
CREATE INDEX ix_bookmarks_deleted
    ON bookmarks (owner_id, deleted_at DESC)
    WHERE deleted_at IS NOT NULL;

-- 在更新书签时自动维护 updated_at
CREATE TRIGGER trg_bookmarks_set_updated_at
BEFORE UPDATE ON bookmarks
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- 将书签表说明写入 PostgreSQL 元数据
COMMENT ON TABLE bookmarks IS '书签主体；保存用户数据，不直接保存网页抓取结果';


-- =============================================================================
-- 5. folder_shares：文件夹共享
-- =============================================================================

-- 保存文件夹访问引用，不复制文件夹和书签数据
CREATE TABLE folder_shares (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- id：共享关系主键；非空且由 PostgreSQL 自动生成
    owner_id            BIGINT NOT NULL,                                 -- owner_id：必填；必须与被共享文件夹的真实所有者一致
    folder_id           BIGINT NOT NULL,                                 -- folder_id：必填；关联被共享文件夹，文件夹删除时级联删除共享关系
    shared_with_user_id BIGINT NOT NULL,                                 -- shared_with_user_id：必填；关联接收者，不能等于 owner_id，同一文件夹不能重复邀请同一用户
    shared_by_user_id   BIGINT NOT NULL,                                 -- shared_by_user_id：必填；关联邀请或权限变更的操作者
    permission          VARCHAR(20) NOT NULL DEFAULT 'viewer',           -- permission：必填；默认 viewer；仅允许 viewer 或 editor
    can_share           BOOLEAN NOT NULL DEFAULT FALSE,                  -- can_share：必填；默认 false；表示接收者能否继续管理分享
    status              VARCHAR(20) NOT NULL DEFAULT 'pending',          -- status：必填；默认 pending；仅允许 pending、active、declined、left 或 revoked，状态迁移由 Rust Service 管理
    invitation_message  VARCHAR(500),                                    -- invitation_message：分享时附带的邀请说明
    accepted_at         TIMESTAMPTZ,                                     -- accepted_at：接受邀请时间；与状态的组合及时间顺序由 Rust Service 校验
    ended_at            TIMESTAMPTZ,                                     -- ended_at：拒绝、退出或撤销时间；与状态的组合及时间顺序由 Rust Service 校验
    created_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,  -- created_at：首次建立共享关系的时间
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,  -- updated_at：共享关系最后更新时间

    -- owner_id、folder_id：必须指向同一个真实文件夹
    CONSTRAINT fk_folder_shares_folder_owner
        FOREIGN KEY (owner_id, folder_id)
        REFERENCES folders (owner_id, id)
        ON DELETE CASCADE,
    -- 接收共享的用户必须存在
    CONSTRAINT fk_folder_shares_target_user FOREIGN KEY (shared_with_user_id) REFERENCES users (id) ON DELETE CASCADE,
    -- 发起共享的用户必须存在
    CONSTRAINT fk_folder_shares_actor FOREIGN KEY (shared_by_user_id) REFERENCES users (id) ON DELETE RESTRICT,
    -- 同一文件夹对同一用户只保存一条共享关系；重新分享时复用该记录
    CONSTRAINT uq_folder_shares_folder_user UNIQUE (folder_id, shared_with_user_id),
    -- 所有者不需要分享给自己
    CONSTRAINT ck_folder_shares_not_owner CHECK (owner_id <> shared_with_user_id),
    -- 初期只提供只读和编辑两种资源权限
    CONSTRAINT ck_folder_shares_permission CHECK (permission IN ('viewer', 'editor')),
    -- status：限制共享生命周期状态
    CONSTRAINT ck_folder_shares_status CHECK (status IN ('pending', 'active', 'declined', 'left', 'revoked')),
    -- 仅 editor 可以被授予继续分享能力
    CONSTRAINT ck_folder_shares_can_share_permission CHECK (NOT can_share OR permission = 'editor')
);

-- 加速查询用户收到的待处理邀请和有效共享
CREATE INDEX ix_folder_shares_shared_with
    ON folder_shares (shared_with_user_id, updated_at DESC)
    WHERE status IN ('pending', 'active');

-- 加速判断文件夹对用户是否存在有效共享
CREATE INDEX ix_folder_shares_folder_active
    ON folder_shares (folder_id, shared_with_user_id)
    WHERE status = 'active';

-- 加速所有者查看自己发出的共享记录
CREATE INDEX ix_folder_shares_owner
    ON folder_shares (owner_id, folder_id, status);

-- 在更新共享关系时自动维护 updated_at
CREATE TRIGGER trg_folder_shares_set_updated_at
BEFORE UPDATE ON folder_shares
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- 将共享表说明写入 PostgreSQL 元数据
COMMENT ON TABLE folder_shares IS '文件夹引用共享；权限自动覆盖共享根的后代树';

-- =============================================================================
-- 6. bookmark_metadata：网页元数据
-- =============================================================================

-- 保存网页抓取结果和简单任务状态，不单独创建任务表
CREATE TABLE bookmark_metadata (
    bookmark_id         BIGINT PRIMARY KEY,                            -- bookmark_id：对应书签 ID；主键兼外键；每条书签最多一份元数据
    fetched_title       VARCHAR(500),                                  -- fetched_title：抓取标题；来自网页，不覆盖书签 title
    fetched_description TEXT,                                          -- fetched_description：抓取描述；来自 meta 或 Open Graph，不覆盖书签 description
    site_name           VARCHAR(300),                                  -- site_name：站点名称；网页声明的网站名
    favicon_source_url  TEXT,                                          -- favicon_source_url：图标来源 URL；网页 favicon 的原始地址
    favicon_storage_key TEXT,                                          -- favicon_storage_key：图标存储键；本地文件路径或对象存储键
    og_image_url        TEXT,                                          -- og_image_url：预览图 URL；Open Graph 图片地址
    content_type        VARCHAR(255),                                  -- content_type：响应类型；网页 HTTP Content-Type
    http_status         SMALLINT,                                      -- http_status：HTTP 状态码；为空或 100-599
    fetch_status        VARCHAR(20) NOT NULL DEFAULT 'pending',        -- fetch_status：抓取状态；默认 pending；仅允许定义的任务状态
    fetch_attempts      INTEGER NOT NULL DEFAULT 0,                    -- fetch_attempts：抓取次数；默认 0；不得为负
    next_fetch_at       TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,-- next_fetch_at：下次抓取时间；用于失败重试
    fetched_at          TIMESTAMPTZ,                                   -- fetched_at：完成时间；最近一次抓取完成的时刻
    error_message       TEXT,                                          -- error_message：错误信息；最近一次抓取失败原因
    created_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,-- created_at：创建时间；记录元数据创建时刻
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,-- updated_at：更新时间；由触发器自动维护

    -- bookmark_id：书签删除时级联删除元数据
    CONSTRAINT fk_bookmark_metadata_bookmark FOREIGN KEY (bookmark_id) REFERENCES bookmarks (id) ON DELETE CASCADE,
    -- fetch_status：限制抓取任务状态
    CONSTRAINT ck_bookmark_metadata_fetch_status CHECK (fetch_status IN ('pending', 'fetching', 'succeeded', 'failed', 'skipped')),
    -- fetch_attempts：抓取次数不得为负
    CONSTRAINT ck_bookmark_metadata_fetch_attempts CHECK (fetch_attempts >= 0),
    -- http_status：非空时必须是有效 HTTP 状态码
    CONSTRAINT ck_bookmark_metadata_http_status CHECK (http_status IS NULL OR http_status BETWEEN 100 AND 599)
);

-- 加速查找当前可以执行的网页抓取任务
CREATE INDEX ix_bookmark_metadata_pending
    ON bookmark_metadata (next_fetch_at, bookmark_id)
    WHERE fetch_status IN ('pending', 'failed');

-- 在更新网页元数据时自动维护 updated_at
CREATE TRIGGER trg_bookmark_metadata_set_updated_at
BEFORE UPDATE ON bookmark_metadata
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- 将网页元数据表说明写入 PostgreSQL 元数据
COMMENT ON TABLE bookmark_metadata IS '网页标题、描述和 favicon；fetch_status 同时驱动简单后台抓取';

-- =============================================================================
-- 7. 字段元数据注释
-- =============================================================================

-- 以下 COMMENT ON COLUMN 将字段说明写入 PostgreSQL，可通过 \d+ 表名查看

-- 7.1 users 字段说明
COMMENT ON COLUMN users.id IS '用户主键；非空且由 PostgreSQL 自动生成';
COMMENT ON COLUMN users.email IS '登录邮箱；必填，去除首尾空格后字符长度必须为 3 到 320；Rust 后台负责存储前规范化为小写；未软删除用户按 lower(btrim(email)) 唯一';
COMMENT ON COLUMN users.password_hash IS '密码哈希；必填，最长 255 个字符；应保存 Argon2id 等安全哈希，禁止保存明文密码';
COMMENT ON COLUMN users.display_name IS '用户显示名称；必填，去除首尾空格后字符长度必须为 1 到 100；Rust 后台负责存储前规范化；不作为登录凭证';
COMMENT ON COLUMN users.system_role IS '系统角色；必填，默认 user；只允许 user 或 admin';
COMMENT ON COLUMN users.status IS '账号状态；必填，默认 active；只允许 pending、active 或 disabled';
COMMENT ON COLUMN users.email_verified_at IS '邮箱验证完成时间；为空表示未验证';
COMMENT ON COLUMN users.last_login_at IS '最近一次成功登录时间';
COMMENT ON COLUMN users.created_at IS '记录创建时间';
COMMENT ON COLUMN users.updated_at IS '记录最后更新时间，由触发器维护';
COMMENT ON COLUMN users.deleted_at IS '软删除时间；为空表示账号正常';

-- 7.2 auth_sessions 字段说明
COMMENT ON COLUMN auth_sessions.id IS '会话主键；非空且由 PostgreSQL 自动生成';
COMMENT ON COLUMN auth_sessions.user_id IS '会话所属用户；必填；用户删除时级联删除其全部会话';
COMMENT ON COLUMN auth_sessions.token_hash IS '随机 Token 的哈希；必填且全表唯一，最长 128 个字符；数据库不保存原始 Token';
COMMENT ON COLUMN auth_sessions.expires_at IS '会话过期时间；必填；Rust 认证服务负责校验其晚于签发时间';
COMMENT ON COLUMN auth_sessions.last_used_at IS '最近一次使用时间';
COMMENT ON COLUMN auth_sessions.revoked_at IS '登出或强制失效时间；为空表示未撤销';
COMMENT ON COLUMN auth_sessions.ip_address IS '创建或最近使用会话的 IP 地址';
COMMENT ON COLUMN auth_sessions.user_agent IS '客户端 User-Agent，用于设备识别';
COMMENT ON COLUMN auth_sessions.created_at IS '会话创建时间';

-- 7.3 folders 字段说明
COMMENT ON COLUMN folders.id IS '文件夹主键；非空且由 PostgreSQL 自动生成';
COMMENT ON COLUMN folders.owner_id IS '文件夹真实所有者；必填；存在文件夹时禁止删除所有者；共享关系不会改变该字段';
COMMENT ON COLUMN folders.parent_id IS '父文件夹；可空表示根文件夹；非空时父文件夹必须属于同一所有者，且不能指向自身';
COMMENT ON COLUMN folders.name IS '文件夹名称；必填，去除首尾空格后长度必须为 1 到 200；Rust 后台负责存储前规范化；同一所有者、同一父级下的未删除名称按 lower(btrim(name)) 唯一';
COMMENT ON COLUMN folders.description IS '文件夹备注';
COMMENT ON COLUMN folders.sort_order IS '同级排序值，推荐使用 1024 间隔排序';
COMMENT ON COLUMN folders.created_at IS '文件夹创建时间';
COMMENT ON COLUMN folders.updated_at IS '文件夹最后更新时间';
COMMENT ON COLUMN folders.deleted_at IS '软删除时间；为空表示正常';

-- 7.4 bookmarks 字段说明
COMMENT ON COLUMN bookmarks.id IS '书签主键；非空且由 PostgreSQL 自动生成';
COMMENT ON COLUMN bookmarks.owner_id IS '书签真实所有者；必填；存在书签时禁止删除所有者';
COMMENT ON COLUMN bookmarks.folder_id IS '所属文件夹；可空表示未分类；非空时文件夹必须属于同一所有者；有书签引用时禁止物理删除文件夹';
COMMENT ON COLUMN bookmarks.title IS '用户设置的标题；必填，去除首尾空格后长度必须为 1 到 500；Rust 后台负责存储前规范化；不会被网页抓取结果覆盖';
COMMENT ON COLUMN bookmarks.url IS '用户保存的 URL；必填，去除首尾空格后字符长度必须为 1 到 8192；Rust 后台负责格式和协议校验；数据库不限制重复 URL';
COMMENT ON COLUMN bookmarks.normalized_url IS '规范化 URL，用于搜索和重复提示，不强制唯一';
COMMENT ON COLUMN bookmarks.description IS '用户填写的简短描述';
COMMENT ON COLUMN bookmarks.sort_order IS '当前文件夹内的排序值';
COMMENT ON COLUMN bookmarks.is_favorite IS '是否收藏';
COMMENT ON COLUMN bookmarks.is_archived IS '是否归档';
COMMENT ON COLUMN bookmarks.created_at IS '书签创建时间';
COMMENT ON COLUMN bookmarks.updated_at IS '书签最后更新时间';
COMMENT ON COLUMN bookmarks.deleted_at IS '软删除时间；为空表示正常';

-- 7.5 folder_shares 字段说明
COMMENT ON COLUMN folder_shares.id IS '共享关系主键；非空且由 PostgreSQL 自动生成';
COMMENT ON COLUMN folder_shares.owner_id IS '被共享文件夹的真实所有者；必填，且必须与 folder_id 对应文件夹的 owner_id 一致';
COMMENT ON COLUMN folder_shares.folder_id IS '被共享的根文件夹；必填；文件夹删除时级联删除共享关系';
COMMENT ON COLUMN folder_shares.shared_with_user_id IS '接收共享的用户；必填且不能等于 owner_id；同一文件夹不能重复邀请同一用户';
COMMENT ON COLUMN folder_shares.shared_by_user_id IS '发起邀请或权限变更的用户；必填；数据库要求用户存在，业务层负责校验其授权';
COMMENT ON COLUMN folder_shares.permission IS '共享权限；必填，默认 viewer；只允许 viewer 或 editor';
COMMENT ON COLUMN folder_shares.can_share IS '接收者是否可以继续管理分享；必填，默认 false';
COMMENT ON COLUMN folder_shares.status IS '共享状态；必填，默认 pending；只允许 pending、active、declined、left 或 revoked';
COMMENT ON COLUMN folder_shares.invitation_message IS '分享时附带的邀请说明';
COMMENT ON COLUMN folder_shares.accepted_at IS '接受邀请时间';
COMMENT ON COLUMN folder_shares.ended_at IS '拒绝、退出或撤销时间';
COMMENT ON COLUMN folder_shares.created_at IS '首次建立共享关系的时间';
COMMENT ON COLUMN folder_shares.updated_at IS '共享关系最后更新时间';

-- 7.6 bookmark_metadata 字段说明
COMMENT ON COLUMN bookmark_metadata.bookmark_id IS '对应书签主键；主键且外键，因此一条书签最多一份元数据；书签删除时级联删除';
COMMENT ON COLUMN bookmark_metadata.fetched_title IS '从网页抓取的标题';
COMMENT ON COLUMN bookmark_metadata.fetched_description IS '从网页 meta 或 Open Graph 抓取的描述';
COMMENT ON COLUMN bookmark_metadata.site_name IS '网站名称';
COMMENT ON COLUMN bookmark_metadata.favicon_source_url IS 'favicon 原始来源 URL';
COMMENT ON COLUMN bookmark_metadata.favicon_storage_key IS 'favicon 本地文件或对象存储键';
COMMENT ON COLUMN bookmark_metadata.og_image_url IS 'Open Graph 预览图 URL';
COMMENT ON COLUMN bookmark_metadata.content_type IS '网页响应 Content-Type';
COMMENT ON COLUMN bookmark_metadata.http_status IS '抓取网页时收到的 HTTP 状态码；可空，非空时必须介于 100 和 599 之间';
COMMENT ON COLUMN bookmark_metadata.fetch_status IS '抓取状态，同时作为简单任务状态；必填，默认 pending；只允许 pending、fetching、succeeded、failed 或 skipped';
COMMENT ON COLUMN bookmark_metadata.fetch_attempts IS '已尝试抓取次数；必填，默认 0；只允许非负整数';
COMMENT ON COLUMN bookmark_metadata.next_fetch_at IS '下次允许抓取时间，用于失败重试';
COMMENT ON COLUMN bookmark_metadata.fetched_at IS '最近一次抓取完成时间；与 fetch_status 的组合及时间顺序由 Rust Service 校验';
COMMENT ON COLUMN bookmark_metadata.error_message IS '最近一次抓取失败原因；与 fetch_status 的组合及清理规则由 Rust Service 校验';
COMMENT ON COLUMN bookmark_metadata.created_at IS '元数据记录创建时间';
COMMENT ON COLUMN bookmark_metadata.updated_at IS '元数据最后更新时间';

COMMIT;
