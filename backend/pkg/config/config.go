package config

import (
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

type Config struct {
	Server struct {
		Port string `yaml:"port"`
		Host string `yaml:"host"`
	} `yaml:"server"`
	Database struct {
		Driver  string `yaml:"driver"`
		DSN     string `yaml:"dsn"`
		MaxOpen int    `yaml:"max_open"`
		MaxIdle int    `yaml:"max_idle"`
	} `yaml:"database"`
	TaskScheduler struct {
		Interval   string `yaml:"interval"`
		MaxWorkers int    `yaml:"max_workers"`
	} `yaml:"task_scheduler"`
	Security struct {
		SecretKey string `yaml:"secret_key"`
	} `yaml:"security"`
}

func LoadConfig(path string) (*Config, error) {
	// Standardkonfiguration laden
	defaultConfig := &Config{
		Server: struct {
			Port string `yaml:"port"`
			Host string `yaml:"host"`
		}{
			Port: "8080",
			Host: "localhost",
		},
		Database: struct {
			Driver  string `yaml:"driver"`
			DSN     string `yaml:"dsn"`
			MaxOpen int    `yaml:"max_open"`
			MaxIdle int    `yaml:"max_idle"`
		}{
			Driver:  "sqlite",
			DSN:     "tasks.db",
			MaxOpen: 10,
			MaxIdle: 5,
		},
		TaskScheduler: struct {
			Interval string `yaml:"interval"`

			MaxWorkers int `yaml:"max_workers"`
		}{
			Interval: "*/30 * * * * *",

			MaxWorkers: 3,
		},
		Security: struct {
			SecretKey string `yaml:"secret_key"`
		}{
			SecretKey: "your-secret-key",
		},
	}

	// Benutzerdefinierte Konfiguration laden
	configPath := getConfigPath(path)
	data, err := os.ReadFile(configPath)
	if err != nil {

		if os.IsNotExist(err) {
			// Datei existiert nicht, erstelle sie mit Standardwerten
			return defaultConfig, saveConfig(configPath, defaultConfig)
		}

		return nil, err
	}

	if err := yaml.Unmarshal(data, defaultConfig); err != nil {
		return nil, err
	}

	return defaultConfig, nil
}

func getConfigPath(path string) string {
	if path != "" {
		return path
	}

	// Suche nach Konfigurationsdatei in verschiedenen Pfaden
	possiblePaths := []string{
		"./config.yaml",
		"./config/config.yaml",
		"/etc/IngestListApiWrapper/config.yaml",
	}

	for _, p := range possiblePaths {
		if _, err := os.Stat(p); err == nil {
			return p
		}
	}

	// Standardpfad
	return "./config/config.yaml"
}

func saveConfig(path string, config *Config) error {
	// Erstelle Verzeichnis falls nicht vorhanden
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return err
	}

	data, err := yaml.Marshal(config)
	if err != nil {
		return err
	}

	return os.WriteFile(path, data, 0644)
}
