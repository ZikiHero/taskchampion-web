package migrations

import (
	"fmt"

	"gorm.io/gorm"
)

type Migration struct {
	ID   string
	Up   func(*gorm.DB) error
	Down func(*gorm.DB) error
}

type SeedConfig struct {
	Email          string
	HashedPassword string
}

func GetAllMigrations(seedConfig SeedConfig) []Migration {
	return []Migration{
		{
			ID: "001_seed_initial_user",
			Up: func(db *gorm.DB) error {
				var count int64
				if err := db.Raw("SELECT COUNT(*) FROM users").Scan(&count).Error; err != nil {
					return err
				}

				if count == 0 {
					return db.Exec(`
                        INSERT INTO users (password, email, created_at, updated_at)
                        VALUES (?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                    `, seedConfig.HashedPassword, seedConfig.Email).Error
				}

				return nil
			},
			Down: func(db *gorm.DB) error {
				return db.Exec(`DELETE FROM users WHERE email = ?`, seedConfig.Email).Error
			},
		},
		{
			ID: "002_create_migration_history",
			Up: func(db *gorm.DB) error {
				return db.Exec(`
                    CREATE TABLE IF NOT EXISTS migration_history (
                        id           INTEGER PRIMARY KEY AUTOINCREMENT,
                        migration_id TEXT NOT NULL UNIQUE,
                        applied_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
                    );
                `).Error
			},
			Down: func(db *gorm.DB) error {
				return db.Exec(`DROP TABLE IF EXISTS migration_history;`).Error
			},
		},
	}
}

func syncUser(db *gorm.DB, seedConfig SeedConfig) error {
	if seedConfig.Email == "" || seedConfig.HashedPassword == "" {
		return nil
	}

	return db.Exec(`
        UPDATE users SET 
            password   = ?,
            email      = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = (SELECT id FROM users LIMIT 1)
    `, seedConfig.HashedPassword, seedConfig.Email).Error
}

func Migrate(db *gorm.DB, seedConfig SeedConfig) error {
	allMigrations := GetAllMigrations(seedConfig)

	if !db.Migrator().HasTable("migration_history") {
		for _, m := range allMigrations {
			if m.ID == "002_create_migration_history" {
				if err := m.Up(db); err != nil {
					return fmt.Errorf("failed to create migration_history: %w", err)
				}
				break
			}
		}
	}

	var appliedMigrations []string
	if err := db.Table("migration_history").
		Pluck("migration_id", &appliedMigrations).Error; err != nil {
		return fmt.Errorf("failed to get migration history: %w", err)
	}

	applied := make(map[string]bool)
	for _, id := range appliedMigrations {
		applied[id] = true
	}

	for _, migration := range allMigrations {
		if applied[migration.ID] {
			continue
		}

		fmt.Printf("Running migration: %s\n", migration.ID)

		if err := db.Transaction(func(tx *gorm.DB) error {
			if err := migration.Up(tx); err != nil {
				return fmt.Errorf("migration %s failed: %w", migration.ID, err)
			}

			return tx.Exec(
				"INSERT INTO migration_history (migration_id) VALUES (?)",
				migration.ID,
			).Error
		}); err != nil {
			return err
		}

		fmt.Printf("Migration %s completed\n", migration.ID)
	}

	// Sync User bei jedem Start
	if err := syncUser(db, seedConfig); err != nil {
		return fmt.Errorf("failed to sync user: %w", err)
	}

	return nil
}

func Rollback(db *gorm.DB, seedConfig SeedConfig) error {
	allMigrations := GetAllMigrations(seedConfig)

	var lastMigration string
	if err := db.Table("migration_history").
		Order("applied_at DESC").
		Limit(1).
		Pluck("migration_id", &lastMigration).Error; err != nil {
		return fmt.Errorf("failed to get last migration: %w", err)
	}

	if lastMigration == "" {
		return fmt.Errorf("no migrations to rollback")
	}

	var migration *Migration
	for i := range allMigrations {
		if allMigrations[i].ID == lastMigration {
			migration = &allMigrations[i]
			break
		}
	}

	if migration == nil {
		return fmt.Errorf("migration %s not found", lastMigration)
	}

	fmt.Printf("Rolling back migration: %s\n", migration.ID)

	return db.Transaction(func(tx *gorm.DB) error {
		if err := migration.Down(tx); err != nil {
			return fmt.Errorf("rollback %s failed: %w", migration.ID, err)
		}

		return tx.Exec(
			"DELETE FROM migration_history WHERE migration_id = ?",
			migration.ID,
		).Error
	})
}
