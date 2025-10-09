# FinanceQuery

Open-source financial data API and web interface.

## Quick Start

```bash
./start.sh
```

Or manually:

```bash
docker-compose up -d
```

Access at: **http://localhost:8080**

> Note: Using port 8080 to avoid conflicts with other services

## Services

- **Frontend**: React web interface with market data visualization
- **Backend**: FastAPI service providing real-time stock data
- **Nginx**: Reverse proxy handling routing

## Features

- Real-time stock quotes and market data
- Historical price charts
- Market indices, sectors, and movers
- Financial news aggregation
- RESTful API with comprehensive documentation
- WebSocket support for live updates

## Configuration

### Custom Domain

1. Copy template: `cp nginx/conf.d/README.template.conf nginx/conf.d/yourdomain.com.conf`
2. Edit and uncomment configuration
3. Place SSL certificates in `nginx/ssl/yourdomain.com/`
4. Restart: `docker-compose restart nginx`

### Environment Variables

Backend (`.env`):
- `LOG_LEVEL` - INFO, DEBUG, WARNING, ERROR
- `USE_SECURITY` - Enable rate limiting
- `REDIS_URL` - Optional caching

Frontend (`frontend/.env`):
- `VITE_API_URL` - Backend API endpoint

## 故障排除

### 502 Bad Gateway 错误

运行诊断工具：
```bash
./diagnose.sh
```

常见解决方法：
1. 等待30-60秒让服务完全启动
2. 检查后端状态：`docker logs financequery-backend`
3. 测试后端：`curl http://localhost:8000/ping`
4. 重启服务：`docker compose restart`

详细故障排除：[TROUBLESHOOTING.md](TROUBLESHOOTING.md)

### Redis相关

Redis是**可选的**，不需要配置。后端会自动使用内存连接管理器。

## License

MIT License - Free and open-source software

This project is an educational tool providing free financial data without corporate affiliation. Compliant with Google Ads policies for open-source tools.

## Disclaimer

Financial data is provided for informational purposes only. Not financial advice. Data accuracy is not guaranteed.
