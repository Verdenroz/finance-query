# 502错误完整诊断指南

## 已修复的问题

### 1. 端口暴露配置 ✅
**问题：** 后端和前端直接暴露端口到宿主机
**修复：** 使用`expose`而非`ports`，只有Nginx对外

```yaml
# 之前（错误）
backend:
  ports:
    - "8000:8000"

# 现在（正确）
backend:
  expose:
    - "8000"
```

### 2. 协议传递问题 ✅
**问题：** `$scheme`始终是`http`，即使外部是`https`
**修复：** 使用`$real_scheme`变量，从`X-Forwarded-Proto`获取

```nginx
# nginx.conf 添加
map $http_x_forwarded_proto $real_scheme {
    default $http_x_forwarded_proto;
    '' $scheme;
}

# default.conf 使用
proxy_set_header X-Forwarded-Proto $real_scheme;
```

### 3. Host头传递 ✅
**问题：** 使用`$host`可能丢失端口信息
**修复：** 改用`$http_host`保留完整Host

```nginx
# 之前
proxy_set_header Host $host;

# 现在
proxy_set_header Host $http_host;
```

### 4. 前端Nginx配置 ✅
**检查：** Frontend容器内部Nginx配置正确
```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

### 5. Docker网络配置 ✅
**检查：** 所有容器在同一网络
```yaml
networks:
  financequery-network:
    driver: bridge
```

## 诊断流程

### 步骤1: 运行诊断脚本

```bash
cd /tmp/cc-agent/58304387/project
./test-nginx-config.sh
```

这会检查：
- ✅ 容器运行状态
- ✅ Nginx配置语法
- ✅ 内部网络连接
- ✅ 外部访问
- ✅ 日志输出

### 步骤2: 手动测试

**A. 检查容器状态**
```bash
docker ps
```
应该看到：
```
financequery-nginx     0.0.0.0:8080->80/tcp
financequery-backend   8000/tcp
financequery-frontend  80/tcp
```

**B. 测试内部连接**
```bash
# 进入Nginx容器
docker exec -it financequery-nginx sh

# 测试前端
wget -O- http://frontend:80

# 测试后端
wget -O- http://backend:8000/ping

# 退出
exit
```

**C. 测试外部访问**
```bash
# 测试前端
curl http://localhost:8080/

# 测试后端API
curl http://localhost:8080/v1/indices
```

**D. 测试直接访问（应该失败）**
```bash
# 这些应该连接失败（正确行为）
curl http://localhost:8000/ping  # ❌ 应该失败
curl http://localhost:80/        # ❌ 应该失败
```

### 步骤3: 检查日志

```bash
# Nginx日志
docker logs financequery-nginx --tail 50

# 后端日志
docker logs financequery-backend --tail 50

# 前端构建日志
docker logs financequery-frontend
```

### 步骤4: 检查防火墙

```bash
# UFW
sudo ufw status

# 应该允许8080
sudo ufw allow 8080/tcp

# iptables
sudo iptables -L -n | grep 8080
```

## 如果本地访问正常但域名502

这说明问题在**反向代理层**！

### 检查反向代理配置

**你的反向代理服务器（lfnrm.xyz）必须：**

1. **指向正确的地址和端口**
```nginx
proxy_pass http://YOUR_DOCKER_HOST_IP:8080;
```
**不是** `http://YOUR_IP:8000` ❌

2. **传递必要的头**
```nginx
proxy_set_header Host $host;
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;  # 重要！https
proxy_set_header X-Forwarded-Host $host;
```

3. **支持WebSocket**
```nginx
proxy_http_version 1.1;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
```

4. **合理的超时**
```nginx
proxy_connect_timeout 60s;
proxy_send_timeout 60s;
proxy_read_timeout 60s;
```

### 完整的反向代理配置示例

```nginx
server {
    listen 443 ssl http2;
    server_name lfnrm.xyz;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        # 指向Docker主机的8080端口
        proxy_pass http://YOUR_DOCKER_HOST_IP:8080;

        # 必须的头
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;  # 注意是https
        proxy_set_header X-Forwarded-Host $host;

        # WebSocket
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # 超时
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # 缓冲
        proxy_buffering off;
        proxy_request_buffering off;
    }
}

# HTTP重定向
server {
    listen 80;
    server_name lfnrm.xyz;
    return 301 https://$server_name$request_uri;
}
```

### 测试反向代理连接

**在反向代理服务器上测试：**
```bash
# 测试能否连接到Docker主机
telnet YOUR_DOCKER_HOST_IP 8080

# 或
nc -zv YOUR_DOCKER_HOST_IP 8080

# 测试HTTP请求
curl -v http://YOUR_DOCKER_HOST_IP:8080/v1/indices
```

**如果失败，检查：**
1. Docker主机防火墙
2. 云服务商安全组
3. 网络连通性

## 常见502原因

### 1. 上游服务未启动
```bash
# 检查
docker ps | grep financequery

# 解决
docker compose restart
```

### 2. 网络不通
```bash
# 在反向代理服务器测试
curl http://YOUR_IP:8080/v1/indices

# 如果超时，检查防火墙
```

### 3. Nginx配置错误
```bash
# 检查语法
docker exec financequery-nginx nginx -t

# 如果错误，查看具体信息
docker logs financequery-nginx
```

### 4. 后端启动慢
```bash
# 等待后端完全启动
sleep 60

# 检查后端健康
curl http://localhost:8080/ping
```

### 5. 反向代理配置错误
```bash
# 在反向代理服务器查看日志
tail -f /var/log/nginx/error.log

# 常见错误信息：
# - "connect() failed (111: Connection refused)"  → 目标端口错误或服务未启动
# - "no resolver defined" → DNS解析问题
# - "upstream timed out" → 超时设置太短
```

## 完整重新部署流程

如果以上都不行，完整重新部署：

```bash
# 1. 停止所有
docker compose down

# 2. 清理（可选）
docker system prune -f

# 3. 确认配置文件正确
cat nginx/nginx.conf | grep "map \$http_x_forwarded_proto"
cat nginx/conf.d/default.conf | grep "proxy_set_header X-Forwarded-Proto \$real_scheme"

# 4. 重新构建
docker compose build --no-cache

# 5. 启动
docker compose up -d

# 6. 等待启动
echo "等待容器启动..."
sleep 60

# 7. 测试
./test-nginx-config.sh

# 8. 如果本地通过，测试域名
curl https://lfnrm.xyz/v1/indices
```

## 调试技巧

### 查看实时请求

**在Docker主机：**
```bash
# Nginx访问日志
docker logs -f financequery-nginx

# 过滤错误
docker logs financequery-nginx 2>&1 | grep -i error
```

**在反向代理服务器：**
```bash
# 实时访问日志
tail -f /var/log/nginx/access.log

# 实时错误日志
tail -f /var/log/nginx/error.log
```

### 抓包分析

```bash
# 在Docker主机抓包
sudo tcpdump -i any -nn port 8080

# 查看是否有请求到达
```

### 测试代理头

```bash
# 创建测试端点查看收到的头
docker exec financequery-backend python3 -c "
from fastapi import FastAPI, Request
import uvicorn

app = FastAPI()

@app.get('/debug-headers')
async def debug(request: Request):
    return dict(request.headers)

uvicorn.run(app, host='0.0.0.0', port=9000)
"

# 通过域名访问查看
curl https://lfnrm.xyz/debug-headers
```

## 检查清单

部署后逐项检查：

- [ ] 容器全部运行：`docker ps`
- [ ] Nginx配置正确：`docker exec financequery-nginx nginx -t`
- [ ] 本地前端可访问：`curl http://localhost:8080/`
- [ ] 本地API可访问：`curl http://localhost:8080/v1/indices`
- [ ] 8080端口已开放：`sudo ufw allow 8080/tcp`
- [ ] 反向代理能连接：`telnet YOUR_IP 8080`（在反向代理服务器）
- [ ] 反向代理配置正确：检查proxy_pass地址
- [ ] 反向代理传递X-Forwarded-Proto: https
- [ ] 域名DNS解析正确：`nslookup lfnrm.xyz`
- [ ] HTTPS证书有效：浏览器访问无证书错误
- [ ] 域名可访问：`curl https://lfnrm.xyz/v1/indices`

## 联系支持

如果所有步骤都完成但仍然502，提供以下信息：

1. 诊断脚本输出：`./test-nginx-config.sh`
2. 容器状态：`docker ps`
3. Nginx日志：`docker logs financequery-nginx --tail 100`
4. 后端日志：`docker logs financequery-backend --tail 100`
5. 反向代理配置文件
6. 反向代理错误日志
7. 网络测试结果：从反向代理服务器`telnet YOUR_IP 8080`

## 总结

502错误的根本原因只有几种：

1. **上游服务不可达** - 检查容器、网络、防火墙
2. **配置错误** - 检查Nginx语法、proxy_pass地址
3. **超时** - 增加超时时间
4. **协议/Host不匹配** - 检查代理头传递

本项目已经修复了所有已知的配置问题。如果仍然502，问题一定在反向代理层或网络连接。

**关键点：**
- 本地通过IP:8080访问正常 ✅
- 域名502 ❌
- **问题在反向代理配置或网络连接** 🎯
