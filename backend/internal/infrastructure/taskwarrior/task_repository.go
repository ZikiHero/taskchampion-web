package taskwarrior

import (
	"context"
	"fmt"
	"log"

	"hufschlaeger.net/tcweb-backend/internal/domain/interfaces"
	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
)

type TaskRepository struct {
	client     *Client
	taskMapper *Mapper
}

func NewTaskRepository(client *Client) interfaces.TaskRepository {
	return &TaskRepository{client: client, taskMapper: NewMapper()}
}

func (t *TaskRepository) Create(ctx context.Context, task *value_objects.Task) error {
	//TODO implement me
	panic("implement me")
}

func (t *TaskRepository) FindByUUID(ctx context.Context, uuid string) (*value_objects.Task, error) {
	var dto TaskDTO
	err := t.client.Get(ctx, "/tasks/"+uuid, &dto)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch tasks: %w", err)
	}

	task, err := t.taskMapper.DTOToDomain(dto)
	return task, err
}

func (t *TaskRepository) FindByType(ctx context.Context, tasks *[]value_objects.Task, taskType string) error {
	//TODO implement me
	panic("implement me")
}

func (t *TaskRepository) FindAll(ctx context.Context) ([]*value_objects.Task, error) {
	var dtos []TaskDTO

	err := t.client.Get(ctx, "/tasks", &dtos)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch tasks: %w", err)
	}
	// Prüfe ob Ergebnis leer ist
	if len(dtos) == 0 {
		return []*value_objects.Task{}, nil
	}

	// Konvertiere DTOs zu Domain Models
	tasksResult := make([]*value_objects.Task, 0, len(dtos))
	errorCount := 0

	for i, dto := range dtos {
		task, err := t.taskMapper.DTOToDomain(dto)
		if err != nil {
			errorCount++
			log.Printf("❌ ERROR converting task %d (UUID: %s): %v", i, dto.UUID, err)
			log.Printf("   DTO was: %+v", dto)
			continue
		}

		tasksResult = append(tasksResult, task)
	}

	return tasksResult, nil
}

func (t *TaskRepository) Update(ctx context.Context, task *value_objects.Task) error {
	requestDto := t.taskMapper.DomainToDTO(task)
	responseDto := TaskDTO{}

	err := t.client.Post(ctx, "/tasks", &requestDto, &responseDto)
	if err != nil {
		return fmt.Errorf("failed to create task: %w", err)
	}

	return nil
}

func (t *TaskRepository) Delete(ctx context.Context, uuid string) error {
	//TODO implement me
	panic("implement me")
}

func (t *TaskRepository) FindPendingTasks(ctx context.Context, tasks *[]value_objects.Task) error {
	//TODO implement me
	panic("implement me")
}

func (t *TaskRepository) Count(ctx context.Context) (int, error) {
	//TODO implement me
	panic("implement me")
}

func (t *TaskRepository) CountByStatus(ctx context.Context, status value_objects.TaskStatus) (int, error) {
	//TODO implement me
	panic("implement me")
}

func (t *TaskRepository) CountByAllStatuses(ctx context.Context) (map[value_objects.TaskStatus]int, error) {
	//TODO implement me
	panic("implement me")
}

func (t *TaskRepository) Sync(ctx context.Context) error {
	err := t.client.Post(ctx, "/sync", nil, nil)
	if err != nil {

		return fmt.Errorf("failed to sync tasks: %w", err)
	}
	return nil
}
