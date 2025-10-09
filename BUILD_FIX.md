# 构建修复说明

## 问题

npm依赖冲突：react-i18next@15.2.3 需要 i18next >= 23.4.0

## 已修复

### 1. 更新 package.json
```json
"dependencies": {
  "i18next": "^23.15.0",      // 从 ^23.17.4 降级
  "react-i18next": "^15.0.0",  // 从 ^15.2.3 降级
}
```

### 2. 更新 Dockerfile
```dockerfile
RUN npm install --legacy-peer-deps
```

## 构建步骤

### 方法1：本地构建（推荐）
```bash
cd frontend

# 删除旧的lock文件和node_modules
rm -rf node_modules package-lock.json

# 安装依赖
npm install --legacy-peer-deps

# 构建
npm run build

# 返回项目根目录
cd ..
```

### 方法2：Docker构建
```bash
# 清理
docker compose down
docker rmi financequery-frontend

# 重新构建
docker compose build frontend

# 启动
docker compose up -d
```

### 方法3：如果仍有问题
```bash
cd frontend

# 使用更稳定的版本
npm install i18next@23.10.0 react-i18next@14.1.0 --save --legacy-peer-deps

# 构建
npm run build
```

## 验证

```bash
# 检查依赖安装成功
cd frontend
npm list i18next react-i18next

# 应该看到：
# ├── i18next@23.15.0
# └── react-i18next@15.0.0

# 测试构建
npm run build

# 检查dist目录
ls -la dist/
```

## 如果还有问题

### 清理npm缓存
```bash
npm cache clean --force
rm -rf ~/.npm
rm -rf node_modules package-lock.json
npm install --legacy-peer-deps
```

### 使用yarn代替npm
```bash
npm install -g yarn
cd frontend
rm -rf node_modules package-lock.json
yarn install
yarn build
```

## 更新摘要

### 修改的文件
- `frontend/package.json` - 更新i18next和react-i18next版本
- `frontend/Dockerfile` - 添加 --legacy-peer-deps 标志

### 兼容版本
- i18next: 23.15.0（稳定）
- react-i18next: 15.0.0（兼容）

所有功能保持不变，只是版本调整以解决依赖冲突。
