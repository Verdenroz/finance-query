# FinanceQuery Project Summary

## Overview
A complete financial data platform with React frontend and FastAPI backend, deployable via Docker.

## Architecture

### Frontend (React + TypeScript + Vite)
- **Pages**:
  - HomePage: Market overview with indices, movers, sectors, and news
  - StockDetailPage: Detailed stock information with charts and tabs
  - APIDocsPage: Comprehensive API documentation

- **Components**:
  - Layout: Main app structure with header and footer
  - SearchBar: Stock symbol search
  - StockChart: Historical price visualization (Recharts)
  - StockTabs: Tabbed interface for Overview, News, Financials, Holders
  - MarketOverview: Market indices display
  - NewsSection: Financial news feed

- **Features**:
  - Responsive design with Tailwind CSS
  - Professional charts with Recharts library
  - Async data loading with proper error handling
  - Clean, modern UI inspired by kabutan.jp

### Backend (FastAPI + Python)
- RESTful API endpoints for:
  - Quotes (detailed and simple)
  - Historical prices
  - Market movers (gainers, losers, actives)
  - Market indices
  - Sectors
  - News
  - Search
  - Holders
  - Financials
  - Earnings transcripts

- WebSocket endpoints for real-time data
- Rate limiting and caching support
- Comprehensive logging

### Infrastructure (Docker + Nginx)
- **Backend Container**: FastAPI service on port 8000
- **Frontend Container**: React SPA served by Nginx
- **Nginx Proxy**: Routes traffic and handles domain configuration
- **Networks**: Isolated Docker network for service communication

## Deployment

### One-Command Start
```bash
./start.sh
```

### Manual Start
```bash
docker-compose up -d
```

### Custom Domain Setup
1. Copy `nginx/conf.d/README.template.conf` to your domain config
2. Uncomment and customize settings
3. Add SSL certificates to `nginx/ssl/yourdomain.com/`
4. Restart services

## Configuration

### Environment Variables

**Backend (.env)**:
- LOG_LEVEL: Logging verbosity
- USE_SECURITY: Enable rate limiting
- REDIS_URL: Optional Redis cache

**Frontend (frontend/.env)**:
- VITE_API_URL: Backend API URL

### Nginx Configuration
- Default HTTP on port 80
- HTTPS support (port 443) via SSL certificates
- WebSocket proxy for real-time connections
- Gzip compression enabled

## Google Ads Compliance

This project is compliant with Google Ads policies:
- Open-source MIT License
- Educational and informational purpose
- Free API without business entity requirements
- Clear disclaimers about data accuracy
- No financial advice provided

## Data Disclaimer

All financial data is provided for informational and educational purposes only. This is not financial advice. Data accuracy is not guaranteed. No corporate affiliation or business entity.

## License

MIT License - Free to use, modify, and distribute
