#!/bin/bash

echo "========================================="
echo "  FinanceQuery 诊断工具"
echo "========================================="
echo ""

# 检查Docker
echo "1. 检查Docker安装..."
if command -v docker &> /dev/null; then
    echo "   ✓ Docker 已安装"
    docker --version
else
    echo "   ✗ Docker 未安装"
    exit 1
fi

echo ""

# 检查Docker Compose
echo "2. 检查Docker Compose..."
if command -v docker compose &> /dev/null; then
    echo "   ✓ Docker Compose 已安装"
else
    echo "   ✗ Docker Compose 未安装"
    exit 1
fi

echo ""

# 检查容器状态
echo "3. 检查容器状态..."
containers=$(docker ps --format "table {{.Names}}\t{{.Status}}" | grep financequery)
if [ -z "$containers" ]; then
    echo "   ✗ 没有运行中的容器"
    echo ""
    echo "   尝试启动服务："
    echo "   docker compose up -d"
else
    echo "$containers"
fi

echo ""

# 检查后端
echo "4. 检查后端服务..."
if docker ps | grep -q financequery-backend; then
    echo "   ✓ 后端容器正在运行"

    # 测试后端API
    if curl -s http://localhost:8000/ping > /dev/null 2>&1; then
        echo "   ✓ 后端API响应正常"
        echo "   响应内容："
        curl -s http://localhost:8000/ping | head -c 200
        echo ""
    else
        echo "   ✗ 后端API无响应"
        echo ""
        echo "   查看后端日志："
        docker logs financequery-backend --tail=20
    fi
else
    echo "   ✗ 后端容器未运行"
fi

echo ""

# 检查前端
echo "5. 检查前端服务..."
if docker ps | grep -q financequery-frontend; then
    echo "   ✓ 前端容器正在运行"
else
    echo "   ✗ 前端容器未运行"
fi

echo ""

# 检查Nginx
echo "6. 检查Nginx..."
if docker ps | grep -q financequery-nginx; then
    echo "   ✓ Nginx容器正在运行"

    # 测试Nginx
    if curl -s http://localhost:8080 > /dev/null 2>&1; then
        echo "   ✓ Nginx响应正常"
    else
        echo "   ✗ Nginx无响应"
        echo ""
        echo "   查看Nginx日志："
        docker logs financequery-nginx --tail=20
    fi
else
    echo "   ✗ Nginx容器未运行"
fi

echo ""

# 检查端口
echo "7. 检查端口占用..."
if command -v lsof &> /dev/null; then
    echo "   端口8000："
    lsof -i :8000 2>/dev/null || echo "   未占用"
    echo "   端口8080："
    lsof -i :8080 2>/dev/null || echo "   未占用"
elif command -v netstat &> /dev/null; then
    echo "   端口8000："
    netstat -tuln | grep 8000 || echo "   未占用"
    echo "   端口8080："
    netstat -tuln | grep 8080 || echo "   未占用"
else
    echo "   无法检查端口占用（需要lsof或netstat）"
fi

echo ""

# 检查网络
echo "8. 检查Docker网络..."
if docker network ls | grep -q financequery-network; then
    echo "   ✓ 网络已创建"
else
    echo "   ✗ 网络不存在"
fi

echo ""
echo "========================================="
echo "  诊断完成"
echo "========================================="
echo ""

# 提供建议
if docker ps | grep -q financequery; then
    echo "服务正在运行。访问："
    echo "  前端: http://localhost:8080"
    echo "  后端: http://localhost:8000"
    echo ""
    echo "查看日志："
    echo "  docker compose logs -f"
else
    echo "服务未运行。启动服务："
    echo "  ./start.sh"
    echo ""
    echo "或手动启动："
    echo "  docker compose up -d"
fi

echo ""
echo "详细故障排除请查看: TROUBLESHOOTING.md"
