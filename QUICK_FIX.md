# 快速修复指南

## 问题
package-lock.json与package.json不同步（Tailwind v4 → v3）

## 解决方案

### 方法1：本地重新生成（推荐）

在你的本地机器（有Node.js 20+的地方）：

```bash
cd frontend

# 删除旧文件
rm -rf node_modules package-lock.json

# 重新生成
npm install

# 提交新的package-lock.json
git add package-lock.json
git commit -m "Regenerate package-lock.json for Tailwind v3"
```

### 方法2：使用Docker临时容器

```bash
# 进入frontend目录
cd frontend

# 使用Node 20临时容器生成lock文件
docker run --rm -v $(pwd):/app -w /app node:20-alpine sh -c "rm -rf node_modules package-lock.json && npm install"

# 检查生成的文件
ls -lh package-lock.json
```

### 方法3：修改Dockerfile（已完成）

Dockerfile已更新为：
```dockerfile
FROM node:20-alpine as build  # Node 18 → 20

WORKDIR /app

COPY package*.json ./
RUN npm install  # npm ci → npm install (自动生成lock)

COPY . .
RUN npm run build
```

## 关键变化

1. **Node版本：18 → 20**
   - Vite 7需要Node 20+
   - React Router 7需要Node 20+

2. **npm ci → npm install**
   - npm ci要求lock文件完全同步
   - npm install会自动更新lock文件

3. **Tailwind：v4 → v3**
   - 从 4.1.14 降级到 3.4.1
   - PostCSS配置也已更新

## 立即部署

```bash
# 停止容器
docker compose down

# 清理前端镜像
docker rmi financequery-frontend

# 重新构建（会自动生成lock文件）
docker compose build frontend

# 如果构建成功，启动所有服务
docker compose up -d

# 等待
sleep 90

# 测试
curl http://localhost:8080/
```

## 验证

构建成功后，检查CSS：
```bash
docker exec financequery-frontend ls -lh /usr/share/nginx/html/assets/*.css
```

应该看到CSS文件有实际大小（不是0字节）。

## 如果仍然失败

### 检查构建日志
```bash
docker compose build frontend 2>&1 | grep -A 10 "npm install"
```

### 清理所有缓存
```bash
docker compose down
docker system prune -af
docker compose build --no-cache
docker compose up -d
```

### 手动进入容器调试
```bash
# 启动临时容器
docker run -it --rm -v $(pwd)/frontend:/app -w /app node:20-alpine sh

# 在容器内
rm -rf node_modules package-lock.json
npm install
npm run build
ls -lh dist/assets/

# 退出
exit
```

## 关键点

- ✅ Node 20支持Vite 7和React Router 7
- ✅ npm install会处理lock文件同步
- ✅ Tailwind v3配置已修复
- ✅ 不再需要8443端口
