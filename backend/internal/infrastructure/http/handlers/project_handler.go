package handlers

import (
	"github.com/gin-gonic/gin"
	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http/handlers/mapper"
)

type ProjectHandler struct {
	projectService *services.TaskwarriorService
	taskMapper     *mapper.TaskMapper
}

func NewProjectHandler(taskService *services.TaskwarriorService) *ProjectHandler {
	return &ProjectHandler{
		projectService: taskService,
		taskMapper:     mapper.NewTaskMapper(),
	}
}

func (h ProjectHandler) GetProjects(c *gin.Context) {

}

func (h ProjectHandler) GetProject(c *gin.Context) {

}
