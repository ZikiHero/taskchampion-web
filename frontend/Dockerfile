FROM docker.io/node:25-alpine AS builder

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

EXPOSE 80
