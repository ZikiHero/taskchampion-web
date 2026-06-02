FROM docker.io/node:26.3.0-alpine3.22 AS builder

WORKDIR /build

# Cache dependencies layer
COPY package*.json .
RUN npm install

# Build application
COPY . .
RUN npm run build -- --configuration production --output-path=dist

# Production stage
FROM nginx:alpine

# Install runtime dependencies (nur was wirklich gebraucht wird)
RUN apk add --no-cache curl tzdata

WORKDIR /usr/share/nginx/html

# Clean default nginx files
RUN rm -rf ./*

# Copy built app
COPY --from=builder /build/dist/browser .
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/docker-entrypoint.sh"]
