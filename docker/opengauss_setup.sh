#!/bin/bash
# OpenGauss 初始化脚本
# 类似于 PostgreSQL 的 sql_setup.sh，但适配 OpenGauss

set -e

echo "Starting OpenGauss setup for GaussDB Rust driver testing..."

# 等待 OpenGauss 完全启动
sleep 30

echo "OpenGauss container should be ready now..."

# 切换到 omm 用户并执行 SQL 命令
su - omm << 'EOF'

# 连接到数据库并执行初始化
gsql -d postgres << 'EOSQL'

-- 创建测试用户（用于不同认证方式），如果不存在的话
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'postgres_user') THEN
        CREATE USER postgres_user WITH PASSWORD 'password';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'md5_user') THEN
        CREATE USER md5_user WITH PASSWORD 'password';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'scram_user') THEN
        CREATE USER scram_user WITH PASSWORD 'password';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'ssl_user') THEN
        CREATE USER ssl_user WITH PASSWORD 'password';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'gaussdb') THEN
        CREATE USER gaussdb WITH PASSWORD 'Gaussdb@123';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'pass_user') THEN
        CREATE USER pass_user WITH PASSWORD 'password';
    END IF;
END
$$;

-- 授予权限
GRANT ALL PRIVILEGES ON DATABASE postgres TO postgres_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO md5_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO scram_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO ssl_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO pass_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO gaussdb;

-- 创建测试数据库（如果不存在）
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_database WHERE datname = 'test_db') THEN
        CREATE DATABASE test_db OWNER postgres_user;
    END IF;
END
$$;

-- 创建测试表和数据（在 postgres 数据库中，避免切换数据库的问题）

CREATE TABLE test_table (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    value INTEGER
);

INSERT INTO test_table (name, value) VALUES 
    ('test1', 1),
    ('test2', 2),
    ('test3', 3);

-- 创建用于通知测试的表
CREATE TABLE notifications_test (
    id SERIAL PRIMARY KEY,
    message TEXT
);

-- 创建一些测试函数和类型（如果 OpenGauss 支持的话）
CREATE OR REPLACE FUNCTION test_function(input_val INTEGER)
RETURNS INTEGER AS $$
BEGIN
    RETURN input_val * 2;
END;
$$ LANGUAGE plpgsql;

-- 创建测试视图
CREATE VIEW test_view AS 
SELECT id, name, value * 2 as double_value 
FROM test_table;

-- 设置一些测试参数
SET timezone = 'UTC';

EOSQL

echo "OpenGauss setup completed successfully!"

EOF

# 修改 pg_hba.conf 以支持不同的认证方式
# 注意：OpenGauss 的配置文件路径可能与 PostgreSQL 不同
PGDATA="/var/lib/opengauss/data"

# 尝试找到正确的数据目录
if [ ! -d "$PGDATA" ]; then
    # 尝试其他可能的路径
    for possible_path in "/gaussdb/data" "/opt/opengauss/data" "/data" "/var/lib/postgresql/data"; do
        if [ -d "$possible_path" ]; then
            PGDATA="$possible_path"
            echo "Found data directory at: $PGDATA"
            break
        fi
    done
fi

if [ -f "$PGDATA/pg_hba.conf" ]; then
    echo "Configuring authentication methods..."
    
    # 备份原始配置
    cp "$PGDATA/pg_hba.conf" "$PGDATA/pg_hba.conf.backup"
    
    # 添加测试用的认证配置
    cat >> "$PGDATA/pg_hba.conf" << 'EOCONF'

# Test authentication configurations for GaussDB Rust driver
# MD5 authentication (widely supported)
host    all             postgres_user   0.0.0.0/0               md5
host    all             postgres_user   ::0/0                   md5

# MD5 authentication
host    all             md5_user        0.0.0.0/0               md5
host    all             md5_user        ::0/0                   md5

# Trust authentication for scram_user (fallback)
host    all             scram_user      0.0.0.0/0               trust
host    all             scram_user      ::0/0                   trust

# SSL authentication
hostssl all             ssl_user        0.0.0.0/0               md5
hostssl all             ssl_user        ::0/0                   md5

# Trust for local connections (for testing)
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust

# Allow gaussdb user for testing
host    all             gaussdb         0.0.0.0/0               md5
host    all             gaussdb         ::0/0                   md5

# Allow pass_user for testing
host    all             pass_user       0.0.0.0/0               md5
host    all             pass_user       ::0/0                   md5

EOCONF

    echo "Authentication configuration updated."
else
    echo "Warning: pg_hba.conf not found at expected location"
fi

# 配置 postgresql.conf（如果存在）
if [ -f "$PGDATA/postgresql.conf" ]; then
    echo "Configuring PostgreSQL parameters..."
    
    # 备份原始配置
    cp "$PGDATA/postgresql.conf" "$PGDATA/postgresql.conf.backup"
    
    # 添加测试配置
    cat >> "$PGDATA/postgresql.conf" << 'EOCONF'

# Test configurations for GaussDB Rust driver
listen_addresses = '*'
port = 5432
max_connections = 100
shared_buffers = 128MB
log_statement = 'all'
log_min_messages = info
ssl = off


# 如果需要 SSL 支持，取消注释以下行
# ssl = on
# ssl_cert_file = 'server.crt'
# ssl_key_file = 'server.key'

EOCONF

    echo "PostgreSQL configuration updated."
fi

echo "OpenGauss setup script completed!"
