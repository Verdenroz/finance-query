# Start with the official Python 3.12 image
FROM python:3.12-slim

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
COPY requirements.txt .

# Install Python packages
RUN pip install --no-cache-dir -r requirements.txt

# Copy the 'src' directory itself, not just its contents
COPY src /app/src

# Copy setup.py
COPY setup.py /app

# Set the working directory
WORKDIR /app

# Run setup.py to cythonize the files
RUN python setup.py build_ext --inplace

# Expose the port FastAPI will run on
EXPOSE 8000

# Set the entry point to run the FastAPI server using uvicorn
CMD ["uvicorn", "src.main:app", "--host", "0.0.0.0", "--port", "8000"]