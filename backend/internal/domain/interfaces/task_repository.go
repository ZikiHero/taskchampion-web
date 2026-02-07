package interfaces

import (
	"context"

	"hufschlager.net/tcweb-backend/internal/domain/models"
)

type TaskRepository interface {
	Create(ctx context.Context, task *models.Task) error
	FindByID(ctx context.Context, id uint) (*models.Task, error)
	FindByType(ctx context.Context, tasks *[]models.Task, taskType string) error
	FindAll(ctx context.Context) ([]models.Task, error)
	Update(ctx context.Context, task *models.Task) error
	Delete(ctx context.Context, id uint) error
	FindPendingTasks(ctx context.Context, tasks *[]models.Task) error
	FindTasksForCleanUp(ctx context.Context) ([]models.Task, error)

	Count(ctx context.Context) (int, error)
	CountByStatus(ctx context.Context, status models.TaskStatus) (int, error)
	CountByAllStatuses(ctx context.Context) (map[models.TaskStatus]int, error)
}
