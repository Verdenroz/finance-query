# Deployment Options

This document outlines various ways to deploy the FinanceQuery API service. This only covers Docker, AWS Lambda, and Render deployments, but the service can also be deployed on any platform that supports Python and FastAPI.

## Dependencies

For the full list of dependencies, see [`requirements.txt`](https://github.com/Verdenroz/finance-query/blob/master/requirements.txt) in the project root.

## Docker Deployment

Docker provides a straightforward way to self-host the FinanceQuery server. A Dockerfile is included in the repository that installs all dependencies and cythonizes project files for optimal performance.

### Build the Docker Image

```bash
docker build -t finance-query .
```

### Run the Docker Container

```bash
docker run -p 8000:8000 finance-query
```

FinanceQuery will now be available on port 8000 at `http://localhost:8000`.

### Docker with Logging Configuration

Configure logging for different environments:

=== "Development"
    ```bash
    # Build and run with development logging
    docker build \
      --build-arg LOG_LEVEL=DEBUG \
      --build-arg LOG_FORMAT=text \
      --build-arg PERFORMANCE_THRESHOLD_MS=500 \
      -t finance-query .
    
    docker run -p 8000:8000 finance-query
    ```

=== "Production"
    ```bash
    # Build with production settings
    docker build \
      --build-arg LOG_LEVEL=INFO \
      --build-arg LOG_FORMAT=json \
      --build-arg PERFORMANCE_THRESHOLD_MS=2000 \
      -t finance-query .
    
    # Run with additional production configuration
    docker run -p 8000:8000 \
      -e REDIS_URL=redis://redis:6379 \
      -e USE_SECURITY=true \
      -e ADMIN_API_KEY=your-secure-key \
      finance-query
    ```

=== "Runtime Configuration"
    ```bash
    # Override logging settings at runtime
    docker run -p 8000:8000 \
      -e LOG_LEVEL=WARNING \
      -e LOG_FORMAT=json \
      -e PERFORMANCE_THRESHOLD_MS=5000 \
      finance-query
    ```

## AWS Lambda Deployment

### Manual Deployment

1. Follow the [AWS Lambda Deployment Guide](https://docs.aws.amazon.com/lambda/latest/dg/lambda-python.html).
2. Configure environment variables in the Lambda console:

   ```env
   LOG_LEVEL=INFO
   LOG_FORMAT=json
   PERFORMANCE_THRESHOLD_MS=3000
   REDIS_URL=your-redis-endpoint
   USE_SECURITY=true
   ADMIN_API_KEY=your-secure-admin-key
   ```

   **Note**: Lambda has higher latency, so consider using `PERFORMANCE_THRESHOLD_MS=3000` or higher.

### Automated Deployment Workflow

You can use the AWS Deployment Workflow from the repository:

1. Provide repository secrets for:
   - `AWS_SECRET_ID`
   - `AWS_SECRET_KEY`

2. Edit the following values in the workflow file:
   - `AWS_REGION`
   - `ECR_REPOSITORY`
   - `FUNCTION_NAME`

## Render Deployment

### Manual Deployment

1. Follow the [Render Deployment Guide](https://render.com/docs/deploy-to-render).
2. The deployment should use the Dockerfile included in the repository.
3. Be sure to override the CMD in the Dockerfile in your Render project settings to:
   ```
   python -m uvicorn src.main:app --host 0.0.0.0 --port $PORT
   ```

4. Configure environment variables in Render dashboard:

   ```env
   LOG_LEVEL=INFO
   LOG_FORMAT=json
   PERFORMANCE_THRESHOLD_MS=2000
   REDIS_URL=your-redis-url
   USE_SECURITY=true
   ADMIN_API_KEY=your-secure-admin-key
   ```

### Automated Deployment Workflow

You can use the Render Deployment Workflow from the repository:

1. Provide the repository secret:
   - `RENDER_DEPLOY_HOOK_URL`

2. The deploy hook URL can be found in the settings of your Render project.

## Monitoring and Logging

### Production Logging Best Practices

For production deployments, use these recommended logging settings:

```env
# Balanced logging for production
LOG_LEVEL=INFO
LOG_FORMAT=json
PERFORMANCE_THRESHOLD_MS=2000
```

### Log Monitoring

#### CloudWatch (AWS Lambda)

AWS Lambda automatically sends logs to CloudWatch. View logs in the AWS Console:

```bash
# View recent logs
aws logs tail /aws/lambda/your-function-name --follow

# Filter for errors
aws logs filter-log-events --log-group-name /aws/lambda/your-function-name \
  --filter-pattern "ERROR"
```

#### Docker Logging

For Docker deployments, configure log drivers:

```bash
# Using JSON file driver with rotation
docker run -p 8000:8000 \
  --log-driver json-file \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  finance-query

# View logs
docker logs -f container-name
```

#### Render Logging

Render provides built-in log streaming in the dashboard. You can also access logs via CLI:

```bash
# Install Render CLI
npm install -g @render-services/cli

# View logs
render logs --service your-service-name --tail
```

### Troubleshooting

#### Common Issues

**High CPU/Memory Usage:**
- Set `LOG_LEVEL=WARNING` to reduce log volume
- Increase `PERFORMANCE_THRESHOLD_MS` to reduce warnings

**Missing Performance Data:**
- Ensure `LOG_FORMAT=json` for structured data
- Lower `PERFORMANCE_THRESHOLD_MS` for more monitoring

**External API Issues:**
- Monitor logs for "External API FAILED" messages
- Check correlation IDs to trace failed requests

For detailed logging configuration and troubleshooting, see the [Logging Documentation](logging.md).
