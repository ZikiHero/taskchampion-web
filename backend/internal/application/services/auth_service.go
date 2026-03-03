package services

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"

	"hufschlaeger.net/tcweb-backend/internal/domain/interfaces"
	"hufschlaeger.net/tcweb-backend/internal/domain/models"
)

var (
	ErrInvalidCredentials = errors.New("invalid credentials")
	ErrUserNotFound       = errors.New("user not found")
	ErrInvalidToken       = errors.New("invalid token")
	ErrExpiredToken       = errors.New("token expired")
	ErrTokenRevoked       = errors.New("token revoked")
)

type AuthService struct {
	userRepo  interfaces.UserRepository
	secretKey string
	tokenTTL  time.Duration
}

func NewAuthService(
	userRepo interfaces.UserRepository,
	secretKey string,
	tokenTTL time.Duration,
) *AuthService {
	return &AuthService{
		userRepo:  userRepo,
		secretKey: secretKey,
		tokenTTL:  tokenTTL,
	}
}

func (s *AuthService) Authenticate(
	ctx context.Context,
	creds models.UserCredentials,
) (*models.User, error) {
	user, err := s.userRepo.FindByEmail(ctx, creds.Email)
	if err != nil {
		return nil, ErrUserNotFound
	}

	if err := s.VerifyPassword(user.Password, creds.Password); err != nil {
		return nil, ErrInvalidCredentials
	}

	var token string
	if token, err = s.generateToken(user.ID); err != nil {
		return nil, ErrInvalidToken
	}

	user.Token = token
	err = s.userRepo.Update(ctx, user)
	if err != nil {
		return nil, ErrInvalidToken
	}

	return user, nil
}

func (s *AuthService) generateToken(userID uint) (string, error) {
	claims := jwt.MapClaims{
		"sub": userID,
		"exp": time.Now().Add(s.tokenTTL).Unix(),
		"iat": time.Now().Unix(),
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString([]byte(s.secretKey))
}

func (s *AuthService) HashPassword(password string) (string, error) {
	hashedBytes, err := bcrypt.GenerateFromPassword(
		[]byte(strings.TrimSpace(password)),
		bcrypt.DefaultCost,
	)
	return string(hashedBytes), err
}

func (s *AuthService) VerifyPassword(storedHash, inputPassword string) error {
	return bcrypt.CompareHashAndPassword(
		[]byte(storedHash),
		[]byte(strings.TrimSpace(inputPassword)),
	)
}

func (s *AuthService) ValidateToken(tokenString string) (uint, error) {
	// 1. Token parsen und Signatur prüfen
	token, err := jwt.Parse(tokenString, func(token *jwt.Token) (any, error) {
		if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, ErrInvalidToken
		}
		return []byte(s.secretKey), nil
	})

	if err != nil {
		if errors.Is(err, jwt.ErrSignatureInvalid) {
			return 0, ErrInvalidToken
		}
		if errors.Is(err, jwt.ErrTokenExpired) {
			return 0, ErrExpiredToken
		}
		return 0, fmt.Errorf("token parsing failed: %w", err)
	}

	if !token.Valid {
		return 0, ErrInvalidToken
	}

	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok {
		return 0, ErrInvalidToken
	}

	// UserID extrahieren
	userIDFloat, ok := claims["sub"].(float64)
	if !ok {
		return 0, ErrInvalidToken
	}
	userID := uint(userIDFloat)

	// 2. Token mit gespeichertem Token vergleichen
	user, err := s.userRepo.FindByID(context.Background(), userID)
	if err != nil {
		return 0, ErrUserNotFound
	}

	// Prüfen, ob der Token noch mit dem gespeicherten übereinstimmt
	if user.Token != tokenString {
		return 0, ErrTokenRevoked
	}

	return userID, nil
}

func (s *AuthService) RevokeToken(ctx context.Context, userID uint) error {
	// User laden
	user, err := s.userRepo.FindByID(ctx, userID)
	if err != nil {
		return fmt.Errorf("user not found: %w", err)
	}

	// Token löschen
	user.Token = ""

	// User aktualisieren
	if err := s.userRepo.Update(ctx, user); err != nil {
		return fmt.Errorf("failed to revoke token: %w", err)
	}

	return nil
}
