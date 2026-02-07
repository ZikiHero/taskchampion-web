package services

import (
	"context"
	"errors"
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

// Beim Speichern:
func (s *AuthService) HashPassword(password string) (string, error) {
	hashedBytes, err := bcrypt.GenerateFromPassword(
		[]byte(strings.TrimSpace(password)),
		bcrypt.DefaultCost,
	)
	return string(hashedBytes), err
}

// Beim Vergleichen:
func (s *AuthService) VerifyPassword(storedHash, inputPassword string) error {
	return bcrypt.CompareHashAndPassword(
		[]byte(storedHash),
		[]byte(strings.TrimSpace(inputPassword)),
	)
}

func (s *AuthService) ValidateToken(tokenString string) (uint, error) {
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
		return 0, err
	}

	if claims, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
		userID := uint(claims["sub"].(float64))
		return userID, nil
	}

	return 0, ErrInvalidToken
}
