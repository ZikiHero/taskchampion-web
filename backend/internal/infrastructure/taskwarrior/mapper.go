package taskwarrior

import (
	"fmt"
	"time"

	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
)

type Mapper struct{}

func NewMapper() *Mapper {
	return &Mapper{}
}

// DTOToDomain konvertiert TaskDTO zu Domain Model
func (m *Mapper) DTOToDomain(dto TaskDTO) (*value_objects.Task, error) {
	// UUID
	uuid, err := value_objects.NewTaskUUID(dto.UUID)
	if err != nil {
		return nil, fmt.Errorf("invalid uuid: %w", err)
	}

	// Description
	description, err := value_objects.NewDescription(dto.Description)
	if err != nil {
		return nil, fmt.Errorf("invalid description: %w", err)
	}

	// Status
	status, err := value_objects.NewTaskStatus(dto.Status)
	if err != nil {
		return nil, fmt.Errorf("invalid status: %w", err)
	}

	// Entry - Parse RFC3339 Zeit
	entry, err := time.Parse(time.RFC3339, dto.Entry)
	if err != nil {
		return nil, fmt.Errorf("invalid entry time '%s': %w", dto.Entry, err)
	}

	// Task erstellen
	task, err := value_objects.NewTask(uuid, description, status, entry)
	if err != nil {
		return nil, fmt.Errorf("failed to create task: %w", err)
	}

	// Modified Zeit setzen
	if dto.Modified != "" {
		modified, err := time.Parse(time.RFC3339, dto.Modified)
		if err != nil {
			return nil, fmt.Errorf("invalid modified time '%s': %w", dto.Modified, err)
		}
		task.SetModified(modified)
	}

	// Tags setzen
	if len(dto.Tags) > 0 {
		for _, tagStr := range dto.Tags {
			tag, err := value_objects.NewTag(tagStr)
			if err != nil {
				// Log Warning aber breche nicht ab
				fmt.Printf("warning: invalid tag '%s': %v\n", tagStr, err)
				continue
			}
			err = task.AddTag(tag)
			if err != nil {
				return nil, err
			}
		}
	}

	// Priority setzen (optional)
	if dto.Priority != nil && *dto.Priority != "" {
		priority, err := value_objects.NewPriority(*dto.Priority)
		if err != nil {
			return nil, fmt.Errorf("invalid priority '%s': %w", *dto.Priority, err)
		}
		task.SetPriority(priority)
	}

	// Due Date setzen (optional)
	if dto.Due != nil && *dto.Due != "" {
		due, err := time.Parse(time.RFC3339, *dto.Due)
		if err != nil {
			return nil, fmt.Errorf("invalid due time '%s': %w", *dto.Due, err)
		}
		task.SetDue(due)
	}

	// Project setzen (optional)
	if dto.Project != nil && *dto.Project != "" {
		project, err := value_objects.NewProject(*dto.Project)
		if err != nil {
			return nil, fmt.Errorf("invalid project '%s': %w", *dto.Project, err)
		}
		task.SetProject(project)
	}

	return task, nil
}

// DomainToDTO konvertiert Domain Model zu DTO
func (m *Mapper) DomainToDTO(task *value_objects.Task) TaskDTO {
	dto := TaskDTO{
		UUID:        task.UUID().String(),
		Description: task.Description().String(),
		Status:      string(task.Status()),
		Entry:       task.Entry().Format(time.RFC3339),
		Modified:    task.Modified().Format(time.RFC3339),
	}

	// Tags
	if len(task.Tags()) > 0 {
		tags := make([]string, 0, len(task.Tags()))
		for _, tag := range task.Tags() {
			tags = append(tags, tag.String())
		}
		dto.Tags = tags
	}

	// Priority (optional)
	if task.Priority() != nil {
		priority := string(*task.Priority())
		dto.Priority = &priority
	}

	// Due (optional)
	if task.Due() != nil {
		due := task.Due().Format(time.RFC3339)
		dto.Due = &due
	}

	// Project (optional)
	if task.Project() != nil {
		project := task.Project().String()
		dto.Project = &project
	}

	return dto
}
