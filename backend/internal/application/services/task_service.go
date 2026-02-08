package services

import (
	"context"
	"fmt"

	"hufschlaeger.net/tcweb-backend/internal/domain/interfaces"
	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
)

type TaskService struct {
	taskRepository interfaces.TaskRepository
}

func NewTaskService(taskRepo interfaces.TaskRepository) *TaskService {
	return &TaskService{
		taskRepository: taskRepo,
	}
}

func (s *TaskService) ListUsers(ctx context.Context, limit, offset int) ([]value_objects.Task, error) {
	tasks, err := s.taskRepository.FindAll(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to list tasks: %w", err)
	}

	return tasks, nil
}
