# 故障排除指南

## 502 Bad Gateway 错误

### 原因
502错误通常表示Nginx无法连接到后端服务。

### 解决步骤

#### 1. 检查容器状态
```bash
docker ps
```

应该看到3个运行中的容器：
- financequery-backend
- financequery-frontend
- financequery-nginx

#### 2. 检查后端日志
```bash
docker logs financequery-backend
```

查找错误信息。常见问题：
- Python依赖缺失
- Cython编译失败
- 端口冲突

#### 3. 检查后端是否响应
```bash
docker exec financequery-backend curl http://localhost:8000/ping
```

或从宿主机：
```bash
curl http://localhost:8000/ping
```

应该返回：
```json
{"status":"healthy","timestamp":"..."}
```

#### 4. 检查Nginx配置
```bash
docker logs financequery-nginx
```

#### 5. 重启服务
```bash
docker compose down
docker compose up -d
```

等待30-60秒让服务完全启动。

## 后端无法启动

### 检查Cython编译
后端需要编译Cython扩展。查看构建日志：

```bash
docker compose build backend
```

### 缺少依赖
确保所有Python依赖都已安装：
```bash
docker compose build --no-cache backend
```

## 前端502错误

### 检查前端构建
```bash
docker logs financequery-frontend
```

前端应该通过Nginx提供静态文件。

### 重新构建前端
```bash
docker compose build --no-cache frontend
docker compose up -d frontend
```

## 网络问题

### 检查Docker网络
```bash
docker network ls
docker network inspect financequery-network
```

### 重建网络
```bash
docker compose down
docker network prune
docker compose up -d
```

## 常见错误修复

### 1. 端口已被占用
```
Error: bind: address already in use
```

**解决方法：** 修改 `docker-compose.yml` 中的端口：
```yaml
ports:
  - "9090:80"  # 使用其他端口
```

### 2. Redis连接失败
Redis是可选的。如果不需要Redis，不要在`.env`中设置`REDIS_URL`。

后端会自动使用内存连接管理器。

### 3. 文件权限问题
```bash
chmod +x start.sh
docker compose down
docker compose up -d
```

### 4. 镜像构建失败
清除所有缓存重新构建：
```bash
docker compose down
docker system prune -a
docker compose build --no-cache
docker compose up -d
```

## 验证服务

### 后端健康检查
```bash
curl http://localhost:8000/health
```

### 获取市场数据
```bash
curl http://localhost:8000/v1/indices
```

### 前端访问
浏览器打开：`http://localhost:8080`

## 查看详细日志

### 实时日志
```bash
# 所有服务
docker compose logs -f

# 仅后端
docker compose logs -f backend

# 仅前端
docker compose logs -f frontend

# 仅Nginx
docker compose logs -f nginx
```

### 最近的日志
```bash
docker compose logs --tail=100 backend
```

## 完全重置

如果所有方法都失败，完全重置：

```bash
# 停止所有容器
docker compose down -v

# 删除所有镜像
docker rmi financequery-backend financequery-frontend

# 清理系统
docker system prune -a

# 重新构建
docker compose build --no-cache

# 启动
docker compose up -d

# 等待启动
sleep 30

# 检查状态
docker ps
docker logs financequery-backend
```

## 性能问题

### 后端响应慢
1. 检查日志中的性能警告
2. 考虑添加Redis缓存
3. 增加资源限制

### 前端加载慢
1. 检查网络连接
2. 确保Nginx gzip已启用
3. 使用CDN（生产环境）

## 需要帮助？

1. 检查所有日志：`docker compose logs`
2. 确认所有容器都在运行：`docker ps`
3. 验证网络连接：`docker network inspect financequery-network`
4. 查看配置文件是否正确

## 常用命令速查

```bash
# 启动
docker compose up -d

# 停止
docker compose down

# 重启
docker compose restart

# 查看状态
docker ps

# 查看日志
docker compose logs -f

# 重新构建
docker compose build --no-cache

# 进入容器
docker exec -it financequery-backend bash
docker exec -it financequery-frontend sh
docker exec -it financequery-nginx sh
```
