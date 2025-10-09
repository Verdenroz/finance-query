# 反向代理配置指南

## 问题场景

你有一个域名（如 `https://lfnrm.xyz`）反向代理到Docker容器的Nginx，但出现502错误。

## 原因分析

### 直接访问 IP 正常
```
浏览器 → http://YOUR_IP:8080 → Nginx容器 → 正常
```

### 通过域名反向代理 502
```
浏览器 → https://lfnrm.xyz → 你的反向代理服务器 → Nginx容器 → 502错误
```

**核心问题：**
- Host头变成了反向代理服务器的地址
- 缺少 X-Forwarded-Host 等代理头
- 前端静态资源路径错误

## 已修复的配置

### 1. Nginx配置更新

`nginx/conf.d/default.conf` 已更新：

**关键修改：**
```nginx
# 使用 $http_host 而不是 $host
proxy_set_header Host $http_host;

# 添加 X-Forwarded-Host
proxy_set_header X-Forwarded-Host $http_host;

# 信任反向代理的IP
real_ip_header X-Forwarded-For;
set_real_ip_from 0.0.0.0/0;

# 禁用缓冲（避免大文件问题）
proxy_buffering off;
```

**$host vs $http_host 区别：**
- `$host`: 规范化的主机名（可能丢失端口）
- `$http_host`: 完整的Host头内容（包含端口）

### 2. 重新部署

```bash
cd /tmp/cc-agent/58304387/project

# 重启Nginx容器加载新配置
docker compose restart nginx

# 或完全重新部署
docker compose down
docker compose up -d
```

## 你的反向代理服务器配置

### 如果使用 Nginx 作为反向代理

在你的 `lfnrm.xyz` 服务器上：

```nginx
server {
    listen 443 ssl http2;
    server_name lfnrm.xyz;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        # 你的Docker主机IP和端口
        proxy_pass http://YOUR_DOCKER_HOST_IP:8080;

        # 重要：传递所有必要的头
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;

        # WebSocket支持
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # 超时设置
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # 禁用缓冲
        proxy_buffering off;
        proxy_request_buffering off;
    }
}

# HTTP重定向到HTTPS
server {
    listen 80;
    server_name lfnrm.xyz;
    return 301 https://$server_name$request_uri;
}
```

### 如果使用 Caddy

```caddy
lfnrm.xyz {
    reverse_proxy YOUR_DOCKER_HOST_IP:8080 {
        header_up Host {http.request.host}
        header_up X-Real-IP {http.request.remote}
        header_up X-Forwarded-For {http.request.remote}
        header_up X-Forwarded-Proto {http.request.scheme}
        header_up X-Forwarded-Host {http.request.host}
    }
}
```

### 如果使用云服务商的反向代理

**Cloudflare、阿里云、腾讯云等：**

1. 确保代理设置中启用了：
   - WebSocket支持
   - 完整的代理头传递
   - 禁用缓存（或针对API路径禁用）

2. 目标地址：`http://YOUR_IP:8080`

3. 代理头配置：
   ```
   Host: 原始Host
   X-Real-IP: 客户端IP
   X-Forwarded-For: 客户端IP
   X-Forwarded-Proto: https
   X-Forwarded-Host: lfnrm.xyz
   ```

## 验证修复

### 1. 重启后测试
```bash
# 在Docker主机上
docker compose restart nginx

# 等待5秒
sleep 5

# 测试本地
curl http://localhost:8080/v1/indices
```

### 2. 通过域名测试
```bash
# 从任意地方
curl https://lfnrm.xyz/v1/indices

# 应该返回JSON数据
```

### 3. 浏览器测试
打开 `https://lfnrm.xyz`，应该看到：
- ✅ 页面正常加载
- ✅ 样式完整显示
- ✅ API数据正常获取

### 4. 检查浏览器控制台
按F12，查看：
- Network标签：所有请求都是200状态
- Console标签：没有CORS错误或404错误

## 前端样式丢失问题

### 原因
前端使用了绝对路径加载资源：
```html
<link rel="stylesheet" href="/assets/index.css">
```

通过域名访问时，这些路径会变成：
```
https://lfnrm.xyz/assets/index.css
```

如果反向代理配置不正确，这些资源会404。

### 解决方案（已应用）

**1. Nginx配置已更新：**
- 使用 `$http_host` 保留完整Host
- 添加 `X-Forwarded-Host` 头
- 前端容器的Nginx能正确处理资源路径

**2. 前端配置：**
`frontend/.env`:
```bash
VITE_API_URL=
```

使用相对路径，让浏览器自动使用当前域名。

## 常见问题

### Q1: 仍然502错误

**检查反向代理服务器日志：**
```bash
# Nginx
tail -f /var/log/nginx/error.log

# Caddy
journalctl -u caddy -f
```

**常见原因：**
- Docker主机防火墙阻止8080端口
- Docker容器未运行
- 反向代理无法连接到Docker主机

### Q2: 前端加载但API失败

**检查CORS：**
浏览器控制台可能显示CORS错误。

**解决：**
确保反向代理传递了正确的Origin头：
```nginx
proxy_set_header Origin $http_origin;
```

### Q3: WebSocket连接失败

**检查：**
- 反向代理是否支持WebSocket
- 是否设置了 `Upgrade` 和 `Connection` 头

**Nginx配置：**
```nginx
proxy_http_version 1.1;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
```

### Q4: 部分资源404

**原因：** 前端路由问题

**解决：** 在反向代理添加：
```nginx
location / {
    proxy_pass http://YOUR_IP:8080;
    # ... 其他配置

    # 重要：不要尝试处理前端路由
    # 让Docker内的Nginx处理
}
```

## 测试清单

部署后检查：

- [ ] 通过IP访问正常：`http://YOUR_IP:8080`
- [ ] 通过域名访问正常：`https://lfnrm.xyz`
- [ ] 前端样式完整加载
- [ ] API请求正常：`https://lfnrm.xyz/v1/indices`
- [ ] WebSocket连接正常（如果有）
- [ ] 浏览器控制台无错误
- [ ] HTTPS证书有效
- [ ] 页面资源加载时间正常

## 性能优化

### 启用Gzip（在反向代理层）

```nginx
gzip on;
gzip_vary on;
gzip_proxied any;
gzip_comp_level 6;
gzip_types text/plain text/css text/xml text/javascript
           application/x-javascript application/xml+rss
           application/javascript application/json;
```

### 启用缓存（静态资源）

```nginx
location ~* \.(jpg|jpeg|png|gif|ico|css|js|svg|woff|woff2|ttf)$ {
    proxy_pass http://YOUR_IP:8080;
    proxy_cache_valid 200 7d;
    expires 7d;
    add_header Cache-Control "public, immutable";
}
```

### 不缓存API

```nginx
location /v1 {
    proxy_pass http://YOUR_IP:8080;
    proxy_no_cache 1;
    proxy_cache_bypass 1;
    add_header Cache-Control "no-store, no-cache, must-revalidate";
}
```

## 监控

### 检查反向代理状态
```bash
# Nginx
nginx -t
systemctl status nginx

# Caddy
caddy validate
systemctl status caddy
```

### 查看访问日志
```bash
# Nginx
tail -f /var/log/nginx/access.log

# Caddy
tail -f /var/log/caddy/access.log
```

### 测试端到端
```bash
# 从外部测试
curl -I https://lfnrm.xyz

# 应该返回 200 OK
```

## 安全建议

1. **限制来源IP**（如果可能）：
```nginx
allow YOUR_OFFICE_IP;
deny all;
```

2. **启用限流**：
```nginx
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

location /v1 {
    limit_req zone=api burst=20;
    proxy_pass http://YOUR_IP:8080;
}
```

3. **隐藏版本信息**：
```nginx
server_tokens off;
```

4. **启用HSTS**：
```nginx
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
```

## 总结

修复清单：
- ✅ 更新 Nginx 配置（使用 $http_host）
- ✅ 添加 X-Forwarded-Host 头
- ✅ 禁用代理缓冲
- ✅ 配置真实IP获取
- ⏳ 重启 Nginx 容器
- ⏳ 配置你的反向代理服务器
- ⏳ 测试通过域名访问

现在重启Nginx容器，然后检查你的反向代理配置！
