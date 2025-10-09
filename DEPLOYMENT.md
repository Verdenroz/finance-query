# Docker 部署指南

## 架构说明

```
外部访问
  ↓
Nginx容器 (端口 8080) ← 唯一对外端口
  ├── /        → Frontend容器:80 (仅内部)
  └── /v1/*    → Backend容器:8000 (仅内部)
```

### 关键设计

**只有Nginx对外暴露端口！**
- ✅ Nginx: 8080 → 外部可访问
- ❌ Backend: 8000 → 仅Docker内部网络
- ❌ Frontend: 80 → 仅Docker内部网络

### 容器说明

1. **financequery-backend**
   - 暴露: 8000 (仅内部)
   - 功能: FastAPI后端

2. **financequery-frontend**
   - 暴露: 80 (仅内部)
   - 功能: React静态文件

3. **financequery-nginx**
   - 端口: 8080 (对外)
   - 功能: 反向代理

## 快速部署

```bash
# 停止旧容器
docker compose down

# 重新构建并启动
docker compose up --build -d

# 等待启动
sleep 30

# 测试（只能通过Nginx访问）
curl http://localhost:8080/v1/indices
```

**访问：** http://localhost:8080

## 端口配置变化

### 之前（错误）
```yaml
backend:
  ports:
    - "8000:8000"  # ❌ 暴露给宿主机
```

### 现在（正确）
```yaml
backend:
  expose:
    - "8000"       # ✅ 仅Docker内部网络
```

## 验证部署

### 1. 检查容器
```bash
docker ps
```

应该看到：
```
financequery-nginx    0.0.0.0:8080->80/tcp    ← 唯一外部端口
financequery-backend  8000/tcp                ← 仅内部
financequery-frontend 80/tcp                  ← 仅内部
```

### 2. 测试访问
```bash
# ✅ 通过Nginx访问API（正确）
curl http://localhost:8080/v1/indices

# ❌ 直接访问后端（应该失败）
curl http://localhost:8000/v1/indices

# ✅ 通过Nginx访问前端（正确）
curl http://localhost:8080/

# 浏览器访问
open http://localhost:8080
```

## 反向代理配置

### 你的域名反向代理 (lfnrm.xyz)

**目标地址：** `http://YOUR_IP:8080`

**Nginx配置示例：**
```nginx
server {
    listen 443 ssl http2;
    server_name lfnrm.xyz;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://YOUR_IP:8080;
        
        # 必须的代理头
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;

        # WebSocket支持
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # 超时和缓冲
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        proxy_buffering off;
    }
}
```

### 防火墙配置

确保开放8080端口：
```bash
# UFW
sudo ufw allow 8080/tcp

# iptables
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT

# 保存规则
sudo iptables-save
```

## 网络流量

### 本地访问
```
浏览器: http://localhost:8080/v1/indices
  ↓
宿主机 8080端口
  ↓
Nginx容器:80
  ↓ (proxy_pass http://backend)
Backend容器:8000
  ↓
返回数据
```

### 域名访问
```
浏览器: https://lfnrm.xyz/v1/indices
  ↓
你的反向代理服务器
  ↓ (proxy_pass http://YOUR_IP:8080)
Docker宿主机 8080端口
  ↓
Nginx容器:80
  ↓
Backend容器:8000
  ↓
返回数据
```

## 故障排除

### 502错误

**原因1：容器启动顺序**
```bash
# 检查容器状态
docker ps

# 重启顺序
docker compose restart backend
sleep 5
docker compose restart frontend
sleep 5
docker compose restart nginx
```

**原因2：反向代理配置错误**
查看：[REVERSE_PROXY_GUIDE.md](REVERSE_PROXY_GUIDE.md)

**原因3：防火墙阻止**
```bash
# 测试端口
telnet YOUR_IP 8080

# 或
nc -zv YOUR_IP 8080
```

### 前端样式丢失

**已修复：**
- ✅ 前端使用相对路径
- ✅ Nginx正确传递Host头
- ✅ X-Forwarded-Host已配置

### 直接访问后端失败

**这是正确的行为！**

后端现在只在Docker内部网络可访问：
- ❌ `http://localhost:8000` - 应该失败
- ✅ `http://localhost:8080/v1/...` - 通过Nginx

## 完整重新部署

```bash
# 1. 停止所有容器
docker compose down

# 2. 清理旧镜像（可选）
docker rmi financequery-backend financequery-frontend

# 3. 重新构建
docker compose build --no-cache

# 4. 启动
docker compose up -d

# 5. 等待
sleep 60

# 6. 测试
curl http://localhost:8080/v1/indices

# 7. 浏览器
open http://localhost:8080
```

## 日志查看

```bash
# 所有日志
docker compose logs -f

# Nginx日志
docker compose logs -f nginx

# 后端日志
docker compose logs -f backend

# 前端构建日志
docker compose logs frontend
```

## 安全优势

通过只暴露Nginx端口：

1. **减少攻击面** - 只有一个入口点
2. **统一日志** - 所有请求经过Nginx
3. **统一限流** - 在Nginx层实现
4. **统一SSL** - 在反向代理层处理
5. **隐藏内部架构** - 外部无法直接访问后端

## 性能监控

```bash
# 容器资源使用
docker stats

# 网络连接
docker exec financequery-nginx netstat -an | grep ESTABLISHED

# Nginx日志分析
docker exec financequery-nginx tail -f /var/log/nginx/access.log
```

## 更多文档

- [REVERSE_PROXY_GUIDE.md](REVERSE_PROXY_GUIDE.md) - 反向代理详细配置
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - 故障排除
- [PORT_CONFIGURATION.md](PORT_CONFIGURATION.md) - 端口说明
- [502错误解决方案.md](502错误解决方案.md) - 502错误专项

## 关键要点

1. **只有Nginx对外** - 前后端只在内部网络
2. **使用expose而非ports** - 限制外部访问
3. **depends_on** - 确保启动顺序
4. **反向代理必须指向8080** - 不是8000
5. **等待启动** - 首次需要30-60秒

现在部署应该完全正常！
