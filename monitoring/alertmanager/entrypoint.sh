#!/bin/sh
# Alertmanager entrypoint: renders the config template with ALERT_WEBHOOK_URL
# (Alertmanager does not expand env vars in its config file).
set -e

TEMPLATE=/etc/alertmanager/alertmanager.yml.tmpl
CONFIG=/tmp/alertmanager.yml

if [ -n "$ALERT_WEBHOOK_URL" ]; then
    RECEIVER=webhook
else
    echo "WARNING: ALERT_WEBHOOK_URL not set - alerts will be routed to the blackhole receiver (no notifications sent)"
    RECEIVER=blackhole
fi

sed "s|{{DEFAULT_RECEIVER}}|$RECEIVER|g" "$TEMPLATE" > "$CONFIG"

# The template keeps `receivers` last so the webhook receiver can be appended.
if [ -n "$ALERT_WEBHOOK_URL" ]; then
    cat >> "$CONFIG" <<EOF
  - name: webhook
    webhook_configs:
      - url: "$ALERT_WEBHOOK_URL"
        send_resolved: true
EOF
fi

exec /bin/alertmanager \
    --config.file="$CONFIG" \
    --storage.path=/alertmanager
