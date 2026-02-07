package middleware

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"

	"hufschlaeger.net/tcweb-backend/internal/application/services"
)

// AuthMiddleware ist ein HTTP-Middleware-Handler für JWT-Authentifizierung
type AuthMiddleware struct {
	authService *services.AuthService // Korrigierter Typ
}

// NewAuthMiddleware erstellt eine neue AuthMiddleware-Instanz
func NewAuthMiddleware(authService *services.AuthService) *AuthMiddleware {
	return &AuthMiddleware{
		authService: authService,
	}
}

// RequireAuth ist ein Middleware-Handler, der Authentifizierung erfordert
func (m *AuthMiddleware) RequireAuth() gin.HandlerFunc {
	return func(c *gin.Context) {
		// 1. Authorization-Header extrahieren
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error": "Authorization header required",
			})
			return
		}

		// 2. Header-Format prüfen
		tokenParts := strings.Split(authHeader, " ")
		if len(tokenParts) != 2 || strings.ToLower(tokenParts[0]) != "bearer" {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error": "Invalid authorization header format",
			})
			return
		}

		// 3. Token validieren
		userID, err := m.authService.ValidateToken(tokenParts[1])
		if err != nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error": err.Error(),
			})
			return
		}

		// 4. UserID im Kontext speichern
		c.Set("userID", userID)
		c.Next()
	}
}
