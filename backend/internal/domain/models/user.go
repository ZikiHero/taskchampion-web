package models

import "time"

type User struct {
	ID        uint      `json:"id"`
	Password  string    `json:"passwod"` // Hashed!
	Email     string    `json:"email"`
	Token     string    `json:"token"`
	Role      string    `json:"role"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

type UserCredentials struct {
	Email    string
	Password string
}
