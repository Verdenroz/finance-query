#!/bin/bash

echo "========================================="
echo "  FinanceQuery - Docker Deployment"
echo "========================================="
echo ""

if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "Error: Docker Compose is not installed"
    exit 1
fi

echo "Building and starting services..."
docker-compose up --build -d

echo ""
echo "Waiting for services to be ready..."
sleep 10

echo ""
echo "========================================="
echo "  Services are running!"
echo "========================================="
echo ""
echo "  Frontend: http://localhost"
echo "  Backend:  http://localhost:8000"
echo ""
echo "  Health:   http://localhost/health"
echo "  API Docs: http://localhost/api-docs"
echo ""
echo "To view logs: docker-compose logs -f"
echo "To stop:      docker-compose down"
echo ""
