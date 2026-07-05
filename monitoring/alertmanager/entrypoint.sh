#!/bin/sh
# Alertmanager entrypoint: renders the config template with whichever notify
# channel(s) are configured (Alertmanager does not expand env vars itself).
set -e

TEMPLATE=/etc/alertmanager/alertmanager.yml.tmpl
CONFIG=/tmp/alertmanager.yml

HAS_WEBHOOK=false
HAS_EMAIL=false
[ -n "$ALERT_WEBHOOK_URL" ] && HAS_WEBHOOK=true
if [ -n "$ALERT_EMAIL_TO" ] && [ -n "$GF_SMTP_HOST" ] && [ -n "$GF_SMTP_USER" ] && [ -n "$GF_SMTP_PASSWORD" ]; then
    HAS_EMAIL=true
fi

if [ "$HAS_WEBHOOK" = true ] || [ "$HAS_EMAIL" = true ]; then
    RECEIVER=notify
else
    echo "WARNING: no ALERT_WEBHOOK_URL or ALERT_EMAIL_TO/GF_SMTP_* configured - alerts will be routed to the blackhole receiver (no notifications sent)"
    RECEIVER=blackhole
fi

sed "s|{{DEFAULT_RECEIVER}}|$RECEIVER|g" "$TEMPLATE" > "$CONFIG"

# The template keeps `receivers` last so the notify receiver's channels can
# be appended below; a receiver can carry both webhook_configs and
# email_configs at once, so both fire if both are configured.
if [ "$HAS_WEBHOOK" = true ] || [ "$HAS_EMAIL" = true ]; then
    echo "  - name: notify" >> "$CONFIG"

    if [ "$HAS_WEBHOOK" = true ]; then
        cat >> "$CONFIG" <<EOF
    webhook_configs:
      - url: "$ALERT_WEBHOOK_URL"
        send_resolved: true
EOF
    fi

    if [ "$HAS_EMAIL" = true ]; then
        cat >> "$CONFIG" <<EOF
    email_configs:
      - to: "$ALERT_EMAIL_TO"
        from: "${GF_SMTP_FROM_ADDRESS:-$GF_SMTP_USER}"
        smarthost: "$GF_SMTP_HOST"
        auth_username: "$GF_SMTP_USER"
        auth_password: "$GF_SMTP_PASSWORD"
        send_resolved: true
EOF
    fi
fi

exec /bin/alertmanager \
    --config.file="$CONFIG" \
    --storage.path=/alertmanager
