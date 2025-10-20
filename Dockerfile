# Start with the official Python 3.12 image
FROM python:3.13.7-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libkrb5-dev \
    libicu-dev \
    zlib1g-dev \
    openssl \
    tar \
    gzip \
    gcc \
    python3-dev \
    && apt-get clean

# Copy requirements.txt
COPY requirements.txt ./

# Install Python packages
RUN pip install --no-cache-dir -r requirements.txt

# Copy project files for building
COPY pyproject.toml setup.py ./
COPY src /app/src

# Copy setup.py again to app directory for later use
COPY setup.py /app

# Set the working directory
WORKDIR /app

# Create target directory structure for compiled extensions
RUN mkdir -p services/indicators/core src/services/indicators/core

# Run setup.py to cythonize the files
RUN python setup.py build_ext --inplace

# Copy compiled extensions to src directory for imports
RUN cp services/indicators/core/*.so src/services/indicators/core/

# Build arguments for configuration with defaults
ARG LOG_LEVEL=INFO
ARG LOG_FORMAT=json
ARG PERFORMANCE_THRESHOLD_MS=2000
ARG DISABLE_LOGO_FETCHING=false
ARG LOGO_TIMEOUT_SECONDS=1
ARG LOGO_CIRCUIT_BREAKER_THRESHOLD=5
ARG LOGO_CIRCUIT_BREAKER_TIMEOUT=300
ARG APP_RATE_LIMIT_PER_DAY=8000

# Set environment variables with defaults
# Logging configuration
ENV LOG_LEVEL=${LOG_LEVEL}
ENV LOG_FORMAT=${LOG_FORMAT}
ENV PERFORMANCE_THRESHOLD_MS=${PERFORMANCE_THRESHOLD_MS}

# Logo fetching configuration
ENV DISABLE_LOGO_FETCHING=${DISABLE_LOGO_FETCHING}
ENV LOGO_TIMEOUT_SECONDS=${LOGO_TIMEOUT_SECONDS}
ENV LOGO_CIRCUIT_BREAKER_THRESHOLD=${LOGO_CIRCUIT_BREAKER_THRESHOLD}
ENV LOGO_CIRCUIT_BREAKER_TIMEOUT=${LOGO_CIRCUIT_BREAKER_TIMEOUT}

# Rate Limiting Configuration:
# Self-deploying users can set their daily rate limit via APP_RATE_LIMIT_PER_DAY.
# If not set, it defaults to 8000 requests per day.
# Example: docker run -e APP_RATE_LIMIT_PER_DAY=15000 my-application-image
ENV APP_RATE_LIMIT_PER_DAY=${APP_RATE_LIMIT_PER_DAY}

# Expose the port FastAPI will run on
EXPOSE 8000

# Set the entry point to run the FastAPI server using uvicorn
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]
