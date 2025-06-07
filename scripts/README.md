# Scripts Directory

这个目录包含了gaussdb-rust项目的各种脚本工具。

## 目录结构

```
scripts/
├── README.md              # 本文档
├── ci-test.sh             # CI测试脚本（GitHub Actions使用）
├── verify-ci.sh           # CI环境验证脚本
└── bat/                   # Windows特定脚本（已忽略）
    ├── setup-port-forward.bat
    ├── cleanup-port-forward.bat
    └── start-opengauss.ps1
```

## 脚本说明

### CI相关脚本

#### `ci-test.sh`
- **用途**: GitHub Actions CI测试主脚本
- **功能**: 
  - 验证CI环境
  - 运行单元测试
  - 运行认证测试
  - 运行核心集成测试
- **使用**: `bash scripts/ci-test.sh`

#### `verify-ci.sh`
- **用途**: CI环境验证脚本
- **功能**:
  - 检查环境变量
  - 验证Docker容器状态
  - 测试数据库连接
  - 检查测试用户
- **使用**: `bash scripts/verify-ci.sh`

### Windows脚本 (bat目录)

> **注意**: bat目录中的Windows特定脚本已添加到.gitignore中，不会被提交到版本控制。

#### `setup-port-forward.bat`
- **用途**: 设置Windows端口转发
- **功能**: 将本地5433端口转发到5432端口
- **使用**: 以管理员身份运行

#### `cleanup-port-forward.bat`
- **用途**: 清理Windows端口转发规则
- **功能**: 删除之前设置的端口转发
- **使用**: 以管理员身份运行

#### `start-opengauss.ps1`
- **用途**: PowerShell启动OpenGauss容器
- **功能**: 
  - 检查Docker服务
  - 启动docker-compose
  - 等待数据库就绪
- **使用**: `powershell -ExecutionPolicy Bypass -File scripts/bat/start-opengauss.ps1`

## 使用说明

### 开发环境设置

1. **Linux/macOS**: 直接使用shell脚本
   ```bash
   chmod +x scripts/*.sh
   ./scripts/verify-ci.sh
   ```

2. **Windows**: 使用bat目录中的脚本
   ```cmd
   # 以管理员身份运行
   scripts\bat\setup-port-forward.bat
   ```

### CI环境

GitHub Actions会自动使用`ci-test.sh`进行测试。

## 权限说明

- Shell脚本需要执行权限: `chmod +x scripts/*.sh`
- Windows脚本需要管理员权限（端口转发）
- PowerShell脚本可能需要执行策略调整

## 注意事项

1. Windows特定脚本不会被提交到版本控制
2. 所有脚本都应该在项目根目录执行
3. 确保Docker环境正常运行
4. CI脚本依赖特定的环境变量配置
