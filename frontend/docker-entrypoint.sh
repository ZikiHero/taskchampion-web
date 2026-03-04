#!/bin/sh
echo "window.ENV = { apiUrl: '${API_URL}' };" > /usr/share/nginx/html/config.js
nginx -g 'daemon off;'
