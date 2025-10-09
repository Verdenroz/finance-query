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
docker compose up --build -d

echo ""
echo "Waiting for services to start..."
echo "(This may take 30-60 seconds on first run)"
sleep 5

echo "Checking backend status..."
for i in {1..12}; do
    if curl -s http://localhost:8000/ping > /dev/null 2>&1; then
        echo "✓ Backend is ready"
        break
    fi
    echo "  Waiting for backend... ($i/12)"
    sleep 5
done

sleep 5

echo ""
echo "========================================="
echo "  Services are running!"
echo "========================================="
echo ""
echo "  Frontend: http://localhost:8080"
echo "  Backend:  http://localhost:8000"
echo ""
echo "  Health:   http://localhost:8080/health"
echo "  API Docs: http://localhost:8080/api-docs"
echo ""
echo "To view logs: docker-compose logs -f"
echo "To stop:      docker-compose down"
echo ""
