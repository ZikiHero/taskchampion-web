package taskwarrior

// TaskDTO repräsentiert einen Task aus der TaskWarrior API
type TaskDTO struct {
	UUID        string   `json:"uuid"`
	Description string   `json:"description"`
	Status      string   `json:"status"`
	Entry       string   `json:"entry"`    // ← String (RFC3339)
	Modified    string   `json:"modified"` // ← String (RFC3339)
	Tags        []string `json:"tags,omitempty"`
	Priority    *string  `json:"priority"` // ← Pointer (kann null sein)
	Due         *string  `json:"due"`      // ← String Pointer (RFC3339, kann null sein)
	Project     *string  `json:"project"`  // ← Pointer (kann null sein!)
}
