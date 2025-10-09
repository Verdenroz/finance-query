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

Access at: **http://localhost**

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

## License

MIT License - Free and open-source software

This project is an educational tool providing free financial data without corporate affiliation. Compliant with Google Ads policies for open-source tools.

## Disclaimer

Financial data is provided for informational purposes only. Not financial advice. Data accuracy is not guaranteed.
