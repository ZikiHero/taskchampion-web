package main

import (
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/persistence"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/persistence/migrations"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/taskwarrior"
	"hufschlaeger.net/tcweb-backend/pkg/config"
)

func main() {
	cfg, err := config.LoadConfig("")
	if err != nil {
		log.Fatalf("Failed to load config file: %v\n", err)
	}

	db, err := persistence.NewDatabase(
		cfg.Database.Driver,
		cfg.Database.DSN,
		cfg.Database.MaxOpen,
		cfg.Database.MaxIdle,
	)

	userConfig := migrations.SeedConfig{
		Email:          getEnvOrDefault("TCWEB_USER_EMAIL", "demo@example.com"),
		HashedPassword: getEnvOrDefault("TCWEB_USER_PASSWORD", "demoPassword"),
	}

	if err != nil {
		log.Fatalf("Failed to initialize the database: %v\n", err)
	}

	// User-Repository erstellen
	userRepo := persistence.NewUserRepository(db)
	// Auth-Service erstellen
	authService := services.NewAuthService(
		userRepo,
		cfg.Security.SecretKey,
		2*time.Hour,
	)

	userConfig.HashedPassword, err = authService.HashPassword(userConfig.HashedPassword)
	if err != nil {
		log.Fatalf("Failed to hash password: %v\n", err)
	}

	err = migrations.Migrate(db, userConfig)
	if err != nil {
		log.Printf("Failed to migrate database: %v\n", err)
		err := migrations.Rollback(db, userConfig)
		if err != nil {
			log.Fatalf("Failed to rollback migration database: %v\n", err)
		}
	}

	// TaskwarriorService erstellen
	client := taskwarrior.NewClient(cfg.Middleware.BaseURL, "")
	taskRepo := taskwarrior.NewTaskRepository(client)
	taskService := services.NewTaskService(taskRepo)

	// HTTP-Server erstellen und starten
	// server := http.NewServer(taskHandler)
	server := http.NewRouter(authService, taskService)
	go func() {
		if err := server.Run(cfg.Server.Host + ":" + cfg.Server.Port); err != nil {
			log.Fatalf("Server error: %v", err)
		}
	}()

	// Auf Shutdown-Signal warten
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit
	log.Println("Shutting down server...")

	log.Println("Server exited")
}

func getEnvOrDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}
