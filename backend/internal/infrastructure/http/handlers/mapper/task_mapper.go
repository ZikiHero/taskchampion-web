package mapper

import (
	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http/handlers/dto"
)

type TaskMapper struct{}

func NewTaskMapper() *TaskMapper {
	return &TaskMapper{}
}

func (m *TaskMapper) ToResponse(task *value_objects.Task) dto.TaskResponse {

	if task == nil {
		return dto.TaskResponse{}
	}

	response := dto.TaskResponse{
		UUID:        task.UUID().String(),
		Description: task.Description().String(),
		Status:      string(task.Status()),
		Entry:       task.Entry().Format("20060102T150405Z"), // TaskWarrior Format
		Modified:    task.Modified().Format("20060102T150405Z"),
		// Tags:        s,
		Priority: "",
		Due:      "",
		Project:  "",
	}

	if priority := task.Priority(); priority != nil {
		response.Priority = string(*priority)
	}

	if due := task.Due(); due != nil {
		response.Due = due.Format("20060102T150405Z")
	}

	if project := task.Project(); project != nil {
		response.Project = project.String()
	}

	return response
}

func (m *TaskMapper) ToResponseList(tasks []*value_objects.Task) []dto.TaskResponse {

	if tasks == nil {
		return nil
	}
	response := make([]dto.TaskResponse, 0, len(tasks))
	for _, task := range tasks {
		response = append(response, m.ToResponse(task))
	}
	return response
}
