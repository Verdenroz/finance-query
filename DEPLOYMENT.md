# FinanceQuery - Docker Deployment

Open-source financial data API and web interface.

## Quick Start

```bash
docker-compose up -d
```

Access the application at `http://localhost`

## Services

- **Frontend**: React web interface (port 80 via Nginx)
- **Backend**: FastAPI service (port 8000)
- **Nginx**: Reverse proxy for both services

## Custom Domain Configuration

1. Copy the Nginx template:
```bash
cp nginx/conf.d/README.template.conf nginx/conf.d/yourdomain.com.conf
```

2. Edit the file and uncomment/customize the server blocks

3. Place SSL certificates in `nginx/ssl/yourdomain.com/`

4. Restart Nginx:
```bash
docker-compose restart nginx
```

## Environment Variables

Backend configuration in `.env`:
- `LOG_LEVEL` - Logging verbosity (default: INFO)
- `USE_SECURITY` - Enable rate limiting (default: False)
- `REDIS_URL` - Optional Redis for caching

Frontend configuration in `frontend/.env`:
- `VITE_API_URL` - Backend API URL

## License

MIT License - Free and open-source
