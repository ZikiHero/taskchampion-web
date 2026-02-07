package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/domain/models"
)

type AuthHandler struct {
	authService *services.AuthService
}

func NewAuthHandler(authService *services.AuthService) *AuthHandler {
	return &AuthHandler{authService: authService}
}

func (h *AuthHandler) Login(c *gin.Context) {
	var creds models.UserCredentials
	if err := c.ShouldBindJSON(&creds); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	token, err := h.authService.Authenticate(c.Request.Context(), creds)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, token)
}

func (h *AuthHandler) GetProfile(c *gin.Context) {
	userID := c.MustGet("userID").(uint)

	// Hier würde die Logik zur Profilabfrage kommen
	c.JSON(http.StatusOK, gin.H{
		"userID":  userID,
		"message": "Authenticated user profile",
	})
}
