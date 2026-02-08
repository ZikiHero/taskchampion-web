package taskwarrior

import "time"

// TaskDTO - Mapping zu Taskwarrior JSON
type TaskDTO struct {
	UUID        string     `json:"uuid"`
	Description string     `json:"description"`
	Status      string     `json:"status"`
	Entry       time.Time  `json:"entry"`
	Modified    time.Time  `json:"modified"`
	Tags        []string   `json:"tags"`
	Priority    *string    `json:"priority"`
	Due         *time.Time `json:"due"`
	Project     string     `json:"project"`
}
