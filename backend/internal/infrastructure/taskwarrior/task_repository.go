package taskwarrior

import (
	"context"

	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
)

type TaskRepository struct {
	client *TaskwarriorClient
}

func (t TaskRepository) Create(ctx context.Context, task *value_objects.Task) error {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) FindByUUID(ctx context.Context, id uint) (*value_objects.Task, error) {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) FindByType(ctx context.Context, tasks *[]value_objects.Task, taskType string) error {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) FindAll(ctx context.Context) ([]value_objects.Task, error) {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) Update(ctx context.Context, task *value_objects.Task) error {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) Delete(ctx context.Context, id uint) error {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) FindPendingTasks(ctx context.Context, tasks *[]value_objects.Task) error {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) Count(ctx context.Context) (int, error) {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) CountByStatus(ctx context.Context, status value_objects.TaskStatus) (int, error) {
	//TODO implement me
	panic("implement me")
}

func (t TaskRepository) CountByAllStatuses(ctx context.Context) (map[value_objects.TaskStatus]int, error) {
	//TODO implement me
	panic("implement me")
}
