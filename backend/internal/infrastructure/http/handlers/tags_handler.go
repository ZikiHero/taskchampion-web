package handlers

import (
	"github.com/gin-gonic/gin"
	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http/handlers/mapper"
)

type TagsHandler struct {
	tagsService *services.TaskwarriorService
	taskMapper  *mapper.TaskMapper
}

func NewTagsHandler(taskService *services.TaskwarriorService) *TagsHandler {
	return &TagsHandler{
		tagsService: taskService,
		taskMapper:  mapper.NewTaskMapper(),
	}
}

func (h *TagsHandler) GetTags(c *gin.Context) {

}
