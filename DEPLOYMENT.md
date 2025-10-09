# Docker 部署指南

## 架构说明

```
浏览器
  ↓
Nginx (端口 8080)
  ├── / → Frontend容器 (React静态文件)
  └── /v1/* → Backend容器 (FastAPI)
```

### 容器说明

1. **financequery-backend** - FastAPI后端服务
   - 端口: 8000 (仅内部网络)
   - 提供所有API端点 `/v1/*`

2. **financequery-frontend** - React前端 + Nginx
   - 端口: 80 (仅内部网络)
   - 提供React静态文件

3. **financequery-nginx** - 反向代理
   - 端口: 8080 (对外) → 80 (内部)
   - 路由规则:
     - `/` → 前端容器
     - `/v1/*` → 后端容器
     - `/health`, `/ping` → 后端容器

## 快速部署

### 一键启动
```bash
./start.sh
```

### 手动部署
```bash
# 构建并启动
docker compose up --build -d

# 等待服务启动
sleep 30

# 检查状态
docker ps
curl http://localhost:8000/ping
```

访问: **http://localhost:8080**

## 关键配置修复

### 前端API配置
前端使用**相对路径**通过Nginx代理访问后端：

`frontend/.env`:
```bash
VITE_API_URL=
```

这样请求会通过Nginx转发：
- 浏览器: `GET /v1/indices`
- Nginx: 转发到 `backend:8000/v1/indices`

### 网络路由
```
http://localhost:8080/        → frontend:80 (React)
http://localhost:8080/v1/*    → backend:8000 (API)
```

## 验证部署

### 1. 检查容器
```bash
docker ps
```

### 2. 测试后端
```bash
curl http://localhost:8000/ping
curl http://localhost:8000/v1/indices
```

### 3. 测试Nginx
```bash
curl http://localhost:8080/v1/indices
```

### 4. 浏览器
打开: `http://localhost:8080`

## 故障排除

### 502错误
```bash
# 1. 运行诊断
./diagnose.sh

# 2. 查看日志
docker logs financequery-backend

# 3. 等待启动
sleep 30

# 4. 重启
docker compose restart
```

详见: [502错误解决方案.md](502错误解决方案.md)

### 重新构建
```bash
docker compose down
docker compose build --no-cache
docker compose up -d
```

## 更多文档

- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - 详细故障排除
- [PORT_CONFIGURATION.md](PORT_CONFIGURATION.md) - 端口配置
- [README.md](README.md) - 项目概览
