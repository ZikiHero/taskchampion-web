package models

import "time"

type (
	TaskStatus string
)

const ()

type Task struct {
	ID          uint       `gorm:"primaryKey"       json:"id"`
	Status      TaskStatus `gorm:"default:Pending"  json:"status"`
	StartedAt   *time.Time `                        json:"started_at"`
	CompletedAt *time.Time `                        json:"completed_at"`
	Output      string     `gorm:"type:text"        json:"output"` // Ausgabe des Programms
	Error       string     `gorm:"type:text"        json:"error"`  // Fehlerausgabe
	CreatedAt   time.Time  `gorm:"autoCreateTime"   json:"created_at"`
	UpdatedAt   time.Time  `gorm:"autoUpdateTime"   json:"updated_at"`
}
