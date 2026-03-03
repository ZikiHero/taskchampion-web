package handlers

import (
	"errors"
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/domain/value_objects"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http/handlers/mapper"
)

type TasksHandler struct {
	taskService *services.TaskwarriorService
	taskMapper  *mapper.TaskMapper
}

func NewTasksHandler(taskService *services.TaskwarriorService) *TasksHandler {
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
	uuid, err := h.extractUUIDFromRequest(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Task UUID is required"})
		return
	}

	task, err := h.taskService.GetTaskById(c.Request.Context(), uuid)
	if err != nil {
		log.Printf("❌ Error: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to fetch task"})
		return
	}

	response := h.taskMapper.ToResponse(task)
	c.JSON(http.StatusOK, response)
}

func (h *TasksHandler) Sync(c *gin.Context) {
	err := h.taskService.SyncRepo(c)
	if err != nil {
		log.Printf("❌ Error: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to sync tasks"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Tasks synced successfully"})
}

func (h *TasksHandler) CreateTask(c *gin.Context) {

	c.JSON(http.StatusOK, gin.H{"message": "Tasks created successfully"})
}

func (h *TasksHandler) UpdateTask(c *gin.Context) {
	var task value_objects.Task

	err := h.taskService.UpdateTask(c.Request.Context(), task)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update task"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Task updated successfully"})
}

func (h *TasksHandler) DeleteTask(c *gin.Context) {
	uuid, err := h.extractUUIDFromRequest(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Task UUID is required"})
		return
	}
	err = h.taskService.DeleteTask(c.Request.Context(), uuid)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to delete tasks"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Task deleted successfully"})
}

func (h *TasksHandler) extractUUIDFromRequest(c *gin.Context) (string, error) {
	id := c.Param("uuid")
	if id == "" {
		return "", errors.New("task UUID is required")
	}
	return id, nil
}
