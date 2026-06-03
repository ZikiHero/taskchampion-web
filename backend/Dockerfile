# Dockerfile
FROM golang:1.26.4-alpine AS builder

WORKDIR /build

# Dependencies
COPY go.mod go.sum ./
RUN go mod download

# Source
COPY . .

# Build
RUN apk update \
    && apk add --no-cache gcc g++ git make build-base sqlite curl\
    && go mod tidy \
    && CGO_ENABLED=1 GOOS=linux go build -o tcweb-backend ./cmd/server

# Runtime
FROM alpine:latest

RUN apk --no-cache add ca-certificates tzdata

WORKDIR /app

COPY --from=builder /build/tcweb-backend .

EXPOSE 8080

CMD ["./tcweb-backend"]

