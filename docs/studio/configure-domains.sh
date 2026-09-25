#!/bin/bash
# Serve an "under development" page over HTTPS for every domain that points
# at this droplet.
#
#   scp docs/studio/configure-domains.sh root@201.79.9.90:/root/
#   ssh root@201.79.9.90 'bash /root/configure-domains.sh'
#
# Safe to re-run. A domain that does not resolve to this machine is skipped
# with a reason rather than attempted: a failed HTTP-01 challenge counts
# against Let's Encrypt's rate limit, and five failures in an hour locks the
# domain out for that hour.
#
# The placeholder is a static file served by nginx directly -- no application,
# no port, no service. When the real site exists, replace the `root` block in
# /etc/nginx/sites-available/<name> with a proxy_pass and reload.

set -euo pipefail
log() { printf '\n=== %s\n' "$*"; }

EMAIL="esan@etamil.in"

# Every domain intended for this box. Ones whose DNS is not ready are skipped
# and reported, so this list does not need editing as DNS lands.
# api.kelir.org is deliberately absent: it is the kElir API, not a placeholder
# page, and deploy/run-deploy.sh plus certbot handle it separately. Adding it
# here would put an "under development" page in front of the API.
#
# ineo.in is hosted elsewhere for now and is to be moved in a few weeks.
DOMAINS=(qos.ae conf.ae kelir.org)

log "prerequisites"
command -v nginx >/dev/null || { DEBIAN_FRONTEND=noninteractive apt-get update -qq; DEBIAN_FRONTEND=noninteractive apt-get -y install nginx >/dev/null; }
command -v certbot >/dev/null || {
    DEBIAN_FRONTEND=noninteractive apt-get update -qq
    DEBIAN_FRONTEND=noninteractive apt-get -y install certbot python3-certbot-nginx >/dev/null
}
certbot --version

MYIP=$(curl -s -4 --max-time 10 https://ifconfig.me || echo unknown)
log "this machine is ${MYIP}"

READY=()
for DOMAIN in "${DOMAINS[@]}"; do
    NAME="${DOMAIN%%.*}"
    RESOLVED=$(getent hosts "$DOMAIN" | awk '{print $1}' | head -1 || true)

    if [ "$RESOLVED" != "$MYIP" ]; then
        printf '  SKIP %-12s resolves to %s\n' "$DOMAIN" "${RESOLVED:-nothing}"
        continue
    fi
    READY+=("$DOMAIN")
    printf '  OK   %-12s\n' "$DOMAIN"

    # --- the placeholder page ----------------------------------------------
    install -d "/var/www/${NAME}"
    if [ ! -f "/var/www/${NAME}/index.html" ]; then
        cat > "/var/www/${NAME}/index.html" <<PAGE
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title>${DOMAIN}</title>
<style>
  :root { color-scheme: light dark; }
  body { margin:0; min-height:100vh; display:grid; place-items:center;
         font:16px/1.6 system-ui, -apple-system, "Segoe UI", sans-serif;
         background:#0e1621; color:#dce9f8; }
  main { max-width:32rem; padding:2rem; text-align:center; }
  h1 { margin:0 0 .25rem; font-size:1.6rem; letter-spacing:.01em; }
  .ta { font-family:"Noto Sans Tamil", Latha, sans-serif; opacity:.75; margin:0 0 1.5rem; }
  p { margin:.5rem 0; opacity:.7; }
  a { color:#4c9be8; }
  hr { border:0; border-top:1px solid #1d2b3a; margin:1.5rem 0; }
</style>
</head>
<body>
<main>
  <h1>${DOMAIN}</h1>
  <p class="ta">உருவாக்கத்தில் உள்ளது</p>
  <p>Under development.</p>
  <hr>
  <p>Built with <a href="https://etamil.in">eTamil</a>.</p>
</main>
</body>
</html>
PAGE
        echo "       placeholder written to /var/www/${NAME}/index.html"
    else
        echo "       /var/www/${NAME}/index.html exists, left alone"
    fi

    # --- nginx, port 80 only; certbot adds :443 and the redirect ------------
    if [ ! -f "/etc/nginx/sites-available/${NAME}" ]; then
        cat > "/etc/nginx/sites-available/${NAME}" <<CONF
# ${DOMAIN}
#
# Static placeholder. To serve the real application instead, replace the
# location block with:
#
#     location / {
#         proxy_pass http://127.0.0.1:<port>;
#         proxy_set_header Host \$host;
#         proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
#         proxy_set_header X-Forwarded-Proto \$scheme;
#     }
#
# then: nginx -t && systemctl reload nginx

server {
    listen 80;
    listen [::]:80;
    server_name ${DOMAIN} www.${DOMAIN};

    root /var/www/${NAME};
    index index.html;

    location / {
        try_files \$uri \$uri/ /index.html;
    }
}
CONF
        echo "       nginx vhost written"
    else
        echo "       nginx vhost exists, left alone"
    fi
    ln -sf "/etc/nginx/sites-available/${NAME}" "/etc/nginx/sites-enabled/${NAME}"
done

if [ ${#READY[@]} -eq 0 ]; then
    log "no domain resolves to this machine - nothing to do"
    exit 0
fi

log "nginx config test"
nginx -t && systemctl reload nginx

# --- certificates -----------------------------------------------------------
# One run per domain so a single failure does not take the others with it.
# www is attempted first and dropped if it does not resolve, because certbot
# fails the whole request if any name in it fails.
for DOMAIN in "${READY[@]}"; do
    if [ -d "/etc/letsencrypt/live/${DOMAIN}" ]; then
        echo "  ${DOMAIN}: certificate already present"
        continue
    fi
    log "certificate for ${DOMAIN}"
    WWW_OK=$(getent hosts "www.${DOMAIN}" | awk '{print $1}' | head -1 || true)
    if [ "$WWW_OK" = "$MYIP" ]; then
        certbot --nginx -d "${DOMAIN}" -d "www.${DOMAIN}" \
            --non-interactive --agree-tos -m "${EMAIL}" --redirect \
            || echo "  certbot failed for ${DOMAIN}"
    else
        echo "  (www.${DOMAIN} does not point here; certificate for the apex only)"
        certbot --nginx -d "${DOMAIN}" \
            --non-interactive --agree-tos -m "${EMAIL}" --redirect \
            || echo "  certbot failed for ${DOMAIN}"
    fi
done

log "renewal"
systemctl enable --now certbot.timer >/dev/null 2>&1 || true
systemctl list-timers certbot.timer --no-pager | head -3
certbot certificates 2>/dev/null | grep -E "Certificate Name|Expiry" | sed 's/^/  /'

log "result"
for DOMAIN in "${DOMAINS[@]}"; do
    printf '  %-14s http=%s  https=%s\n' "$DOMAIN" \
        "$(curl -s -o /dev/null -m 15 -w '%{http_code}' "http://${DOMAIN}/" || echo '---')" \
        "$(curl -s -o /dev/null -m 15 -w '%{http_code}' "https://${DOMAIN}/" || echo '---')"
done

cat <<'NEXT'

=== notes ===

A domain marked SKIP above did not resolve to this droplet:

  ineo.in     hosted elsewhere for now; add it to DOMAINS and re-run once
              its A record points here.

api.kelir.org is not in this list on purpose. It resolves here and needs a
certificate, but it serves the kElir API rather than a placeholder page:

    bash /root/run-deploy.sh                       # the API on 127.0.0.1:8080
    install -m 644 /opt/kelir/deploy/kelir.nginx.conf         /etc/nginx/sites-available/kelir
    ln -sf /etc/nginx/sites-available/kelir /etc/nginx/sites-enabled/kelir
    nginx -t && systemctl reload nginx
    certbot --nginx -d api.kelir.org

Re-running after DNS changes only does the work that is still missing.

To replace a placeholder with a real site, edit the location block in
/etc/nginx/sites-available/<name> as described in its header comment.
NEXT
