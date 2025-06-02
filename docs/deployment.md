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

## AWS Lambda Deployment

### Manual Deployment

1. Follow the [AWS Lambda Deployment Guide](https://docs.aws.amazon.com/lambda/latest/dg/lambda-python.html).
2. Remember to add the necessary environment variables to the Lambda function.

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

### Automated Deployment Workflow

You can use the Render Deployment Workflow from the repository:

1. Provide the repository secret:
   - `RENDER_DEPLOY_HOOK_URL`

2. The deploy hook URL can be found in the settings of your Render project.
