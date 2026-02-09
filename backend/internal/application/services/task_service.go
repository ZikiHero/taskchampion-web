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

func (s *TaskService) ListTasks(ctx context.Context) ([]*value_objects.Task, error) {
	tasks, err := s.taskRepository.FindAll(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to list tasks: %w", err)
	}

	return tasks, nil
}

func (s *TaskService) GetTaskById(ctx context.Context, uuid string) (*value_objects.Task, error) {
	task, err := s.taskRepository.FindByUUID(ctx, uuid)
	if err != nil {
		return nil, fmt.Errorf("failed to get task: %w", err)
	}

	return task, nil
}
