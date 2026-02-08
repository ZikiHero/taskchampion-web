package main

import (
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/persistence"

	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http"
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
	if err != nil {
		log.Fatalf("Failed to initialize the database: %v\n", err)
	}

	// Repository erstellen
	userRepo := persistence.NewUserRepository(db)

	// Service erstellen

	// Worker erstellen (max. Anzahl v. parallelen Tasks)
	authService := services.NewAuthService(
		userRepo,
		cfg.Security.SecretKey,
		2*time.Hour,
	)

	// HTTP-Server erstellen und starten
	// server := http.NewServer(taskHandler)
	server := http.NewRouter(authService)
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
