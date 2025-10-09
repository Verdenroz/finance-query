# FinanceQuery - Quick Start Guide

## Deploy with One Command

```bash
./start.sh
```

Then open: **http://localhost**

## What You Get

### Web Interface
- **Homepage**: Live market data, indices, top movers, sectors, news
- **Stock Pages**: Search any symbol to see detailed data, charts, financials
- **API Docs**: Complete API reference with examples

### API Endpoints
All accessible at `http://localhost:8000/v1/`

Key endpoints:
- `/v1/quotes?symbols=AAPL,NVDA,TSLA`
- `/v1/historical?symbol=AAPL&range=1mo&interval=1d`
- `/v1/gainers?count=50`
- `/v1/losers?count=50`
- `/v1/actives?count=50`
- `/v1/indices`
- `/v1/sectors`
- `/v1/news?symbol=AAPL`

## Using with Your Domain

### 1. Configure Domain
```bash
cd nginx/conf.d
cp README.template.conf yourdomain.com.conf
```

### 2. Edit Configuration
Uncomment and update `server_name` to your domain

### 3. Add SSL Certificates
Place your certificates in `nginx/ssl/yourdomain.com/`:
- `fullchain.pem`
- `privkey.pem`

### 4. Restart
```bash
docker-compose restart nginx
```

## Environment Configuration

### Backend
Edit `.env` file:
```env
LOG_LEVEL=INFO
USE_SECURITY=true
REDIS_URL=redis://redis:6379  # Optional
```

### Frontend
Edit `frontend/.env`:
```env
VITE_API_URL=https://yourdomain.com
```

Then rebuild:
```bash
docker-compose up --build -d
```

## Monitoring

### View Logs
```bash
docker-compose logs -f
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f nginx
```

### Check Health
```bash
curl http://localhost/health
curl http://localhost/ping
```

### Container Status
```bash
docker-compose ps
```

## Stopping Services

```bash
docker-compose down
```

To remove volumes:
```bash
docker-compose down -v
```

## Updating

1. Pull latest changes
2. Rebuild:
```bash
docker-compose up --build -d
```

## Troubleshooting

### Port Already in Use
Change ports in `docker-compose.yml`:
```yaml
ports:
  - "8080:80"  # Use 8080 instead of 80
```

### Backend Not Responding
Check backend logs:
```bash
docker-compose logs backend
```

### Frontend Not Loading
1. Check if backend is running
2. Verify `VITE_API_URL` in `frontend/.env`
3. Rebuild frontend:
```bash
docker-compose up --build frontend
```

## Support

- Check `DEPLOYMENT.md` for detailed setup
- Review `PROJECT_SUMMARY.md` for architecture
- Visit API docs at http://localhost/api-docs

## License

MIT License - Free and open-source
