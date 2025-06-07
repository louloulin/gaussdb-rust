#!/bin/bash
# OpenGauss CI环境初始化脚本

set -e

echo "🔧 开始配置OpenGauss测试环境..."

# 等待OpenGauss启动
echo "⏳ 等待OpenGauss启动..."
until gsql -U gaussdb -d postgres -c '\q' 2>/dev/null; do
  echo "等待数据库启动..."
  sleep 2
done

echo "✅ OpenGauss已启动，开始配置..."

# 创建测试用户
echo "👥 创建测试用户..."
gsql -U gaussdb -d postgres << 'EOSQL'
-- 创建测试用户
DO $$
BEGIN
    -- pass_user (明文密码)
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'pass_user') THEN
        CREATE USER pass_user WITH PASSWORD 'password';
        GRANT CONNECT ON DATABASE postgres TO pass_user;
        GRANT USAGE ON SCHEMA public TO pass_user;
        GRANT CREATE ON SCHEMA public TO pass_user;
        GRANT ALL PRIVILEGES ON SCHEMA public TO pass_user;
        RAISE NOTICE 'Created user: pass_user';
    END IF;
    
    -- md5_user (MD5认证)
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'md5_user') THEN
        CREATE USER md5_user WITH PASSWORD 'password';
        GRANT CONNECT ON DATABASE postgres TO md5_user;
        GRANT USAGE ON SCHEMA public TO md5_user;
        GRANT CREATE ON SCHEMA public TO md5_user;
        GRANT ALL PRIVILEGES ON SCHEMA public TO md5_user;
        RAISE NOTICE 'Created user: md5_user';
    END IF;
    
    -- scram_user (SCRAM-SHA-256认证)
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'scram_user') THEN
        CREATE USER scram_user WITH PASSWORD 'password';
        GRANT CONNECT ON DATABASE postgres TO scram_user;
        GRANT USAGE ON SCHEMA public TO scram_user;
        GRANT CREATE ON SCHEMA public TO scram_user;
        GRANT ALL PRIVILEGES ON SCHEMA public TO scram_user;
        RAISE NOTICE 'Created user: scram_user';
    END IF;
    
    -- 确保postgres用户权限
    GRANT ALL PRIVILEGES ON DATABASE postgres TO postgres;
    
    -- 确保gaussdb用户权限
    GRANT ALL PRIVILEGES ON DATABASE postgres TO gaussdb;
END
$$;

-- 创建一些测试需要的扩展 (如果支持的话)
DO $$
BEGIN
    -- 尝试创建hstore扩展
    BEGIN
        CREATE EXTENSION IF NOT EXISTS hstore;
        RAISE NOTICE 'Created extension: hstore';
    EXCEPTION WHEN OTHERS THEN
        RAISE NOTICE 'hstore extension not available: %', SQLERRM;
    END;
    
    -- 尝试创建citext扩展
    BEGIN
        CREATE EXTENSION IF NOT EXISTS citext;
        RAISE NOTICE 'Created extension: citext';
    EXCEPTION WHEN OTHERS THEN
        RAISE NOTICE 'citext extension not available: %', SQLERRM;
    END;
END
$$;

-- 显示创建的用户
SELECT 'User Summary:' as info;
SELECT usename, usecreatedb, usesuper, userepl 
FROM pg_user 
WHERE usename IN ('pass_user', 'md5_user', 'scram_user', 'postgres', 'gaussdb')
ORDER BY usename;

-- 显示数据库版本
SELECT version() as database_version;

-- 测试连接
SELECT 'OpenGauss test environment setup completed successfully!' as status;
EOSQL

echo "✅ OpenGauss测试环境配置完成！"
echo "📊 测试用户："
echo "   - pass_user (password认证)"
echo "   - md5_user (md5认证)"  
echo "   - scram_user (scram-sha-256认证)"
echo "   - postgres (trust认证)"
echo "   - gaussdb (sha256认证)"
