package interfaces

import (
	"context"

	"hufschlaeger.net/tcweb-backend/internal/domain/models"
)

type UserRepository interface {
	FindByID(ctx context.Context, id uint) (*models.User, error)
	FindByEmail(ctx context.Context, email string) (*models.User, error)
	Create(ctx context.Context, user *models.User) error
	Update(ctx context.Context, user *models.User) error
	Delete(ctx context.Context, id uint) error
}
