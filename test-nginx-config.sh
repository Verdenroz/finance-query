#!/bin/bash

echo "========================================"
echo "Nginx 配置诊断工具"
echo "========================================"
echo ""

# 检查Docker是否运行
if ! command -v docker &> /dev/null; then
    echo "❌ Docker未安装或不可用"
    exit 1
fi

echo "1️⃣  检查容器状态..."
echo "----------------------------------------"
docker ps --filter "name=financequery" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
echo ""

# 检查Nginx配置语法
echo "2️⃣  检查Nginx配置语法..."
echo "----------------------------------------"
if docker exec financequery-nginx nginx -t 2>&1; then
    echo "✅ Nginx配置语法正确"
else
    echo "❌ Nginx配置语法错误"
    exit 1
fi
echo ""

# 测试内部连接
echo "3️⃣  测试内部网络连接..."
echo "----------------------------------------"

# 测试前端容器
echo "测试 frontend:80..."
if docker exec financequery-nginx wget -q --spider http://frontend:80; then
    echo "✅ Frontend容器可访问"
else
    echo "❌ Frontend容器不可访问"
fi

# 测试后端容器
echo "测试 backend:8000..."
if docker exec financequery-nginx wget -q --spider http://backend:8000/ping; then
    echo "✅ Backend容器可访问"
else
    echo "❌ Backend容器不可访问"
fi
echo ""

# 测试外部访问
echo "4️⃣  测试外部访问..."
echo "----------------------------------------"

# 测试通过Nginx访问前端
echo "测试 localhost:8080/ (前端)..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/ | grep -q "200"; then
    echo "✅ 前端可通过Nginx访问"
else
    echo "❌ 前端无法通过Nginx访问"
fi

# 测试通过Nginx访问后端
echo "测试 localhost:8080/v1/indices (后端API)..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/v1/indices)
if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ 后端API可通过Nginx访问 (HTTP $HTTP_CODE)"
else
    echo "⚠️  后端API响应: HTTP $HTTP_CODE"
fi
echo ""

# 查看最近的Nginx日志
echo "5️⃣  最近的Nginx访问日志..."
echo "----------------------------------------"
docker logs financequery-nginx --tail 10 2>&1 | grep -E "(error|warn|GET|POST)" || echo "无日志"
echo ""

# 检查端口占用
echo "6️⃣  检查端口占用..."
echo "----------------------------------------"
if lsof -i :8080 &> /dev/null 2>&1; then
    echo "8080端口占用情况："
    lsof -i :8080 2>&1 | head -5
else
    if netstat -tlnp 2>/dev/null | grep -q ":8080"; then
        echo "8080端口占用情况："
        netstat -tlnp 2>/dev/null | grep ":8080"
    else
        echo "⚠️  8080端口未被占用（可能Docker未启动）"
    fi
fi
echo ""

# 完整测试
echo "7️⃣  完整端到端测试..."
echo "----------------------------------------"
echo "测试API响应内容："
RESPONSE=$(curl -s http://localhost:8080/v1/indices)
if [ -n "$RESPONSE" ]; then
    echo "$RESPONSE" | head -c 200
    echo "..."
    echo "✅ 获取到数据"
else
    echo "❌ 未获取到数据"
fi
echo ""

echo "========================================"
echo "诊断完成"
echo "========================================"
echo ""
echo "如果所有测试通过，但域名仍然502，问题在于："
echo "1. 你的反向代理服务器配置"
echo "2. 防火墙阻止外部访问8080端口"
echo "3. Docker主机IP地址配置错误"
echo ""
echo "建议执行："
echo "  - 在反向代理服务器上: telnet YOUR_IP 8080"
echo "  - 检查防火墙规则: sudo ufw status"
echo "  - 查看反向代理日志"
