#!/bin/bash

# deploys a vite SPA to specified knet subdomain for demo purposes, NODE_ENV=development by default

# Usage: ./initsubdomain.sh [subdomain]

REMOTE_USER='kenny'
REMOTE_HOST='knet'
SSH_NICKNAME='knet'
$SUBDOMAIN=$(jq -r '.name' package.json)

if [[ -n $1 ]]; then
  SUBDOMAIN=$1
fi

FILENAME_VHOST="$SUBDOMAIN.kennybaron.net"

FILEPATH_NGINX_SITES_AVAILABLE="/etc/nginx/sites-available/$FILENAME_VHOST"
FILEPATH_NGINX_SITES_ENABLED="/etc/nginx/sites-enabled/$FILENAME_VHOST"

# BUILD
# npm run build

echo "server {
  listen [::]:443 ssl; # ipv6only=on;
  listen 443 ssl;

  server_name $FILENAME_VHOST;

  index index.html index.htm index.nginx-debian.html;

  root /var/www/$FILENAME_VHOST/html;

  location / {
          # First attempt to serve request as file, then
          # as directory, then fall back to displaying a 404.
          try_files \$uri \$uri/ =404;
  }

  ssl_certificate /etc/letsencrypt/live/kennybaron.net-0001/fullchain.pem; # managed by Certbot
  ssl_certificate_key /etc/letsencrypt/live/kennybaron.net-0001/privkey.pem; # managed by Certbot
      include /etc/letsencrypt/options-ssl-nginx.conf;
      ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;
}

server {
  listen 80;
  listen [::]:80;

  server_name $FILENAME_VHOST;

  return 301 https://\$host\$request_uri;
}" >$FILENAME_VHOST

# CONFIG & ENABLE NGINX
scp "$FILENAME_VHOST" "$REMOTE_USER"@"$REMOTE_HOST":/tmp
ssh "$REMOTE_USER"@"$REMOTE_HOST" \
  "sudo -S mv /tmp/$FILENAME_VHOST $FILEPATH_NGINX_SITES_AVAILABLE \
  && sudo -S ln -s $FILEPATH_NGINX_SITES_AVAILABLE $FILEPATH_NGINX_SITES_ENABLED \
  && sudo -S service nginx reload"
