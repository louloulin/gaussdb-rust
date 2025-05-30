#!/bin/bash
# OpenGauss 初始化脚本
# 类似于 PostgreSQL 的 sql_setup.sh，但适配 OpenGauss

set -e

echo "Starting OpenGauss setup for GaussDB Rust driver testing..."

# 等待 OpenGauss 完全启动
sleep 10

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
END
$$;

-- 创建测试数据库
CREATE DATABASE test_db OWNER postgres_user;

-- 授予权限
GRANT ALL PRIVILEGES ON DATABASE postgres TO postgres_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO md5_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO scram_user;
GRANT ALL PRIVILEGES ON DATABASE postgres TO ssl_user;
GRANT ALL PRIVILEGES ON DATABASE test_db TO postgres_user;

-- 创建测试表和数据
\c test_db postgres_user

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

if [ -f "$PGDATA/pg_hba.conf" ]; then
    echo "Configuring authentication methods..."
    
    # 备份原始配置
    cp "$PGDATA/pg_hba.conf" "$PGDATA/pg_hba.conf.backup"
    
    # 添加测试用的认证配置
    cat >> "$PGDATA/pg_hba.conf" << 'EOCONF'

# Test authentication configurations for GaussDB Rust driver
# SHA256 authentication (if supported by OpenGauss)
host    all             postgres_user   0.0.0.0/0               sha256
host    all             postgres_user   ::0/0                   sha256

# MD5 authentication
host    all             md5_user        0.0.0.0/0               md5
host    all             md5_user        ::0/0                   md5

# SCRAM-SHA-256 authentication (if supported)
host    all             scram_user      0.0.0.0/0               scram-sha-256
host    all             scram_user      ::0/0                   scram-sha-256

# SSL authentication
hostssl all             ssl_user        0.0.0.0/0               md5
hostssl all             ssl_user        ::0/0                   md5

# Trust for local connections (for testing)
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust

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
