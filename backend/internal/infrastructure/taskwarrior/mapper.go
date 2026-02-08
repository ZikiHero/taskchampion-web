package taskwarrior

import (
	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
)

type Mapper struct{}

func NewMapper() *Mapper {
	return &Mapper{}
}

// DTOToDomain - Konvertierung von DTO zu Domain Model
func (m *Mapper) DTOToDomain(dto TaskDTO) (*value_objects.Task, error) {
	// UUID
	uuid, err := value_objects.NewTaskUUID(dto.UUID)
	if err != nil {
		return nil, err
	}

	// Description
	description, err := value_objects.NewDescription(dto.Description)
	if err != nil {
		return nil, err
	}

	// Status
	status, err := value_objects.NewTaskStatus(dto.Status)
	if err != nil {
		return nil, err
	}

	// Task erstellen
	task, err := value_objects.NewTask(uuid, description, status, dto.Entry)
	if err != nil {
		return nil, err
	}

	// Tags
	for _, tagStr := range dto.Tags {
		tag, err := value_objects.NewTag(tagStr)
		if err != nil {
			continue // Oder Error behandeln
		}
		err = task.AddTag(tag)
		if err != nil {
			return nil, err
		}
	}

	// Priority (optional)
	if dto.Priority != nil {
		priority, err := value_objects.NewPriority(*dto.Priority)
		if err == nil {
			task.SetPriority(priority)
		}
	}

	// Due (optional)
	if dto.Due != nil {
		task.SetDue(*dto.Due)
	}

	// Project (optional)
	if dto.Project != "" {
		project, err := value_objects.NewProject(dto.Project)
		if err == nil {
			task.SetProject(project)
		}
	}

	return task, nil
}

func (m *Mapper) DomainToDTO(task *value_objects.Task) TaskDTO {
	dto := TaskDTO{
		UUID:        task.UUID().String(),
		Description: task.Description().String(),
		Status:      string(task.Status()),
		Entry:       task.Entry(),
		Modified:    task.Modified(),
		Tags:        make([]string, 0),
	}

	// Tags
	for _, tag := range task.Tags() {
		dto.Tags = append(dto.Tags, tag.String())
	}

	// Priority (optional)
	if priority := task.Priority(); priority != nil {
		p := string(*priority)
		dto.Priority = &p
	}

	// Due (optional)
	if due := task.Due(); due != nil {
		dto.Due = due
	}

	// Project (optional)
	if project := task.Project(); project != nil {
		dto.Project = project.String()
	}

	return dto
}
