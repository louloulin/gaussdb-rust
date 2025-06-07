#!/bin/bash
# 简单的CI验证脚本

set -e

echo "🔍 验证CI环境..."

# 检查环境变量
echo "📋 环境变量检查:"
echo "DATABASE_URL: ${DATABASE_URL:-未设置}"
echo "GAUSSDB_HOST: ${GAUSSDB_HOST:-未设置}"
echo "GAUSSDB_PORT: ${GAUSSDB_PORT:-未设置}"

# 检查Docker容器
echo "🐳 Docker容器状态:"
docker ps

# 检查数据库连接
echo "🔌 数据库连接测试:"
if docker exec opengauss-ci gsql -U gaussdb -d postgres -c "SELECT 'CI环境验证成功' as status;" 2>/dev/null; then
    echo "✅ 数据库连接正常"
else
    echo "❌ 数据库连接失败"
    exit 1
fi

# 检查用户
echo "👥 测试用户检查:"
docker exec opengauss-ci gsql -U gaussdb -d postgres -c "
SELECT usename, usecreatedb, usesuper 
FROM pg_user 
WHERE usename IN ('gaussdb', 'pass_user', 'md5_user', 'scram_user')
ORDER BY usename;
" || echo "用户查询失败"

echo "✅ CI环境验证完成"
