#!/bin/bash

# deploys a vite SPA to project name as a knet subdomain for demo purposes, NODE_ENV=development by default
# may specify subdomain word as arg

# Usage: ./deploy.sh [subdomain]

npm run build

REMOTE_USER='kenny'
REMOTE_HOST='knet'
SSH_NICKNAME='knet'
SUBDOMAIN=$(jq -r '.name' package.json)
EXCLUDE_ASSETS=false

if [[ -n $1 ]]; then
  if [[ $1 == "--exclude-assets" || $1 == "-xa" ]]; then
    EXCLUDE_ASSETS=true
  else
    SUBDOMAIN=$1
  fi
fi

if [[ -n $2 ]]; then
  if [[ $2 == "--exclude-assets" || $2 == "-xa" ]]; then
    EXCLUDE_ASSETS=true
  fi
fi

DIR_SERVE=/var/www/$SUBDOMAIN.kennybaron.net/html

echo "Deploying to subdomain $SUBDOMAIN..."

ssh "$REMOTE_USER"@"$REMOTE_HOST" "rm -rf /tmp/$SUBDOMAIN \
  && mkdir -p /tmp/$SUBDOMAIN/assets/"

if $EXCLUDE_ASSETS; then
  scp dist/index.html "$REMOTE_USER"@"$REMOTE_HOST":"/tmp/$SUBDOMAIN"
  scp dist/assets/index-*.js "$REMOTE_USER"@"$REMOTE_HOST":"/tmp/$SUBDOMAIN/assets/"
  scp dist/assets/index-*.css "$REMOTE_USER"@"$REMOTE_HOST":"/tmp/$SUBDOMAIN/assets/"
else
  scp -r dist/* "$REMOTE_USER"@"$REMOTE_HOST":"/tmp/$SUBDOMAIN"
fi

if $EXCLUDE_ASSETS; then
  ssh "$REMOTE_USER"@"$REMOTE_HOST" \
    "sudo -S rm -f $DIR_SERVE/assets/index-*.js \
    && sudo rm -f $DIR_SERVE/assets/index-*.css \
    && sudo mv /tmp/$SUBDOMAIN/index.html $DIR_SERVE \
    && sudo mv /tmp/$SUBDOMAIN/assets/index-*.js $DIR_SERVE/assets \
    && sudo mv /tmp/$SUBDOMAIN/assets/index-*.css $DIR_SERVE/assets \
    && rm -rf /tmp/$SUBDOMAIN"
else
  ssh "$REMOTE_USER"@"$REMOTE_HOST" \
    "sudo -S rm -rf $DIR_SERVE \
    && sudo mkdir -p $DIR_SERVE \
    && sudo mv /tmp/$SUBDOMAIN/* $DIR_SERVE \
    && rm -rf /tmp/$SUBDOMAIN"
fi
