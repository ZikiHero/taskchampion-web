package interfaces

import (
	"context"

	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
)

type TaskRepository interface {
	Create(ctx context.Context, task *value_objects.Task) error
	FindByUUID(ctx context.Context, id uint) (*value_objects.Task, error)
	FindByType(ctx context.Context, tasks *[]value_objects.Task, taskType string) error
	FindAll(ctx context.Context) ([]value_objects.Task, error)
	Update(ctx context.Context, task *value_objects.Task) error
	Delete(ctx context.Context, id uint) error
	FindPendingTasks(ctx context.Context, tasks *[]value_objects.Task) error

	Count(ctx context.Context) (int, error)
	CountByStatus(ctx context.Context, status value_objects.TaskStatus) (int, error)
	CountByAllStatuses(ctx context.Context) (map[value_objects.TaskStatus]int, error)
}
