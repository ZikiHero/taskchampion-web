package services

import (
	"context"
	"fmt"

	"hufschlaeger.net/tcweb-backend/internal/domain/interfaces"
	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
)

type TaskwarriorService struct {
	taskRepository interfaces.TaskRepository
}

func NewTaskService(taskRepo interfaces.TaskRepository) *TaskwarriorService {
	return &TaskwarriorService{
		taskRepository: taskRepo,
	}
}

func (s *TaskwarriorService) ListTasks(ctx context.Context) ([]*value_objects.Task, error) {
	tasks, err := s.taskRepository.FindAll(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to list tasks: %w", err)
	}

	return tasks, nil
}

func (s *TaskwarriorService) GetTaskById(ctx context.Context, uuid string) (*value_objects.Task, error) {
	task, err := s.taskRepository.FindByUUID(ctx, uuid)
	if err != nil {
		return nil, fmt.Errorf("failed to get task: %w", err)
	}

	return task, nil
}

func (s *TaskwarriorService) SyncRepo(ctx context.Context) error {
	return s.taskRepository.Sync(ctx)
}

func (s *TaskwarriorService) UpdateTask(c context.Context, task value_objects.Task) error {
	return s.taskRepository.Update(c, &task)
}

func (s *TaskwarriorService) DeleteTask(ctx context.Context, uuid string) error {
	return s.taskRepository.Delete(ctx, uuid)
}
