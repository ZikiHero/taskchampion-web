package persistence

import (
	"fmt"
	"log"
	"sync"

	"gorm.io/driver/postgres"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"

	"hufschlaeger.net/tcweb-backend/internal/domain/models"
)

var (
	dbInstance *gorm.DB
	dbOnce     sync.Once
)

func NewDatabase(driver, dsn string, maxOpen, maxIdle int) (*gorm.DB, error) {
	var err error
	dbOnce.Do(func() {
		var db *gorm.DB

		switch driver {
		case "postgres":
			db, err = gorm.Open(postgres.Open(dsn), &gorm.Config{
				Logger: logger.Default.LogMode(logger.Info),
			})
		case "sqlite":
			db, err = gorm.Open(sqlite.Open(dsn), &gorm.Config{
				Logger: logger.Default.LogMode(logger.Info),
			})
		default:
			err = fmt.Errorf("unsupported database driver: %s", driver)
			return
		}

		if err != nil {
			return
		}

		// Connection Pool Einstellungen
		sqlDB, err := db.DB()
		if err != nil {
			return
		}

		sqlDB.SetMaxOpenConns(maxOpen)
		sqlDB.SetMaxIdleConns(maxIdle)
		sqlDB.SetConnMaxLifetime(0) // 0 = unbegrenzt

		// AutoMigrate for persistence models only (exclude value objects)
		if err := db.AutoMigrate(
			&models.User{},
		); err != nil {
			log.Printf("Warning: AutoMigrate failed: %v", err)
		}

		dbInstance = db
	})

	if err != nil {
		return nil, fmt.Errorf("database initialization failed: %w", err)
	}

	return dbInstance, nil
}
