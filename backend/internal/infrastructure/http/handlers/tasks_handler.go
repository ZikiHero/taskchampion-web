package handlers

import (
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http/handlers/mapper"
)

type TasksHandler struct {
	taskService *services.TaskService
	taskMapper  *mapper.TaskMapper
}

func NewTasksHandler(taskService *services.TaskService) *TasksHandler {
	return &TasksHandler{
		taskService: taskService,
		taskMapper:  mapper.NewTaskMapper(),
	}
}

func (h *TasksHandler) GetAllTasks(c *gin.Context) {
	tasks, err := h.taskService.ListTasks(c.Request.Context())
	if err != nil {
		log.Printf("❌ Error: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch tasks"})
		return
	}

	response := h.taskMapper.ToResponseList(tasks)
	c.JSON(http.StatusOK, response)
}

func (h *TasksHandler) GetTask(c *gin.Context) {
	id := c.Param("uuid")
	if id == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Task UUID is required"})
		return
	}

	task, err := h.taskService.GetTaskById(c.Request.Context(), id)
	if err != nil {
		log.Printf("❌ Error: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch task"})
		return
	}

	response := h.taskMapper.ToResponse(task)
	c.JSON(http.StatusOK, response)
}
