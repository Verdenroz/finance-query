# Start with the AWS Lambda Python 3.12 image
FROM public.ecr.aws/lambda/python:3.12

# Install system dependencies
RUN microdnf install -y \
    ca-certificates \
    krb5-libs \
    libicu \
    zlib \
    openssl \
    tar \
    gzip \
    && microdnf clean all

# Download and install .NET 8.0
RUN curl -SL --output dotnet.tar.gz https://dotnetcli.azureedge.net/dotnet/Runtime/8.0.0/dotnet-runtime-8.0.0-linux-x64.tar.gz \
    && dotnet_sha512='1a8b1826318cbf6a25b1e7673f8d7027898d745c6b9c033c9d6f664c1040cd26e7a0172c4c589b5598a2a6a69610ad6f76f24a6f3b3bfd8f52b492b3c44e3d6f' \
    && mkdir -p /usr/share/dotnet \
    && tar -zxf dotnet.tar.gz -C /usr/share/dotnet \
    && rm dotnet.tar.gz \
    && ln -s /usr/share/dotnet/dotnet /usr/bin/dotnet

# Copy requirements.txt
COPY requirements.txt .

# Install Python packages
RUN pip install -r requirements.txt

# Copy the 'src' directory itself, not just its contents
COPY src ${LAMBDA_TASK_ROOT}/src

# Set the entry point for the AWS Lambda function
CMD ["src/main.handler"]