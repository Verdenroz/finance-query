#!/bin/sh
# Caddy entrypoint with automatic password hashing
set -e

# Hash the monitoring password if provided
if [ -n "$MONITORING_PASSWORD" ]; then
    echo "Hashing monitoring password..."
    HASHED_PASSWORD=$(caddy hash-password --plaintext "$MONITORING_PASSWORD")
    export HASHED_PASSWORD
    echo "Password hashed successfully"
else
    echo "WARNING: MONITORING_PASSWORD not set, authentication will not work!"
    exit 1
fi

# Replace placeholder in Caddyfile with hashed password
sed "s|{{HASHED_PASSWORD}}|$HASHED_PASSWORD|g" /etc/caddy/Caddyfile > /tmp/Caddyfile
export CADDYFILE=/tmp/Caddyfile

# Start Caddy with the modified config
exec caddy run --config /tmp/Caddyfile --adapter caddyfile
