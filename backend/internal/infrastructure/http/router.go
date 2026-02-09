package http

import (
	"log"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"

	"hufschlaeger.net/tcweb-backend/internal/application/services"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http/handlers"
	"hufschlaeger.net/tcweb-backend/internal/infrastructure/http/middleware"
)

/*
| Method       | Endpoint                  | Description                           |
|--------------|---------------------------|---------------------------------------|
| **System**   |                           |                                       |
| `GET`        | `/health`                 | Service health check                  |
| `POST`       | `/sync`                   | Manually trigger synchronization      |
| **Tasks**    |                           |                                       |
| `GET`        | `/tasks`                  | List all tasks (can be filtered by `tag` or `project`) |
| `POST`       | `/tasks`                  | Create a new task                     |
| `GET`        | `/tasks/:uuid`            | Get details of a specific task        |
| `PUT`        | `/tasks/:uuid`            | Update an existing task               |
| `DELETE`     | `/tasks/:uuid`            | Delete a task                         |
| **Projects** |                           |                                       |
| `GET`        | `/projects`               | List all unique projects              |
| `GET`        | `/projects/:name`         | Get stats for a project               |
| `GET`        | `/projects/:name/details` | Get detailed project info             |
| `GET`        | `/projects/:name/tasks`   | List all tasks in a project           |
| `POST`       | `/projects/:name/tasks`   | Create a new task in a project        |
| **Tags**     |                           |                                       |
| `GET`        | `/tags`                   | List all unique tags                  |
| `GET`        | `/tags/:name`             | Get stats for a tag                   |
| `GET`        | `/tags/:name/details`     | Get detailed tag info                 |
| `GET`        | `/tags/:name/tasks`       | List all tasks with a tag             |
| `POST`       | `/tags/:name/tasks`       | Create a new task with a specific tag |
*/
func NewRouter(
	authService *services.AuthService,
	taskService *services.TaskwarriorService,
) *gin.Engine {
	router := gin.Default()

	router.ForwardedByClientIP = true
	if err := router.SetTrustedProxies([]string{"*"}); err != nil {
		log.Printf("Can not set trusted proxies.\n")
	}
	corsConfig := cors.DefaultConfig()
	corsConfig.AllowOrigins = []string{"*"}
	corsConfig.AllowHeaders = []string{"Origin", "Content-Type", "Authorization"}
	corsConfig.AllowMethods = []string{"GET", "POST"}
	// It's important that the cors configuration is used before declaring the routes.
	router.Use(cors.New(corsConfig))

	// middleware
	authMiddleware := middleware.NewAuthMiddleware(authService)
	// Handler initialisieren
	authHandler := handlers.NewAuthHandler(authService)
	taskHandler := handlers.NewTasksHandler(taskService)
	projectHandler := handlers.NewProjectHandler(taskService)
	tagshandler := handlers.NewTagsHandler(taskService)
	// public endpoints
	api := router.Group("/api")
	{
		api.POST("/login", authHandler.Login)
	}

	// potected endpoints
	authenticated := api.Group("/")
	authenticated.Use(authMiddleware.RequireAuth())
	{
		authenticated.POST("/logout", authHandler.Logout)

		api.GET("/tasks", taskHandler.GetAllTasks)
		api.POST("/tasks", taskHandler.CreateTask)
		api.GET("/tasks/:uuid", taskHandler.GetTask)
		api.PUT("/tasks/:uuid", taskHandler.UpdateTask)
		api.DELETE("/tasks/:uuid", taskHandler.DeleteTask)

		api.GET("/projects", projectHandler.GetProjects)
		api.GET("/projects/:uuid", projectHandler.GetProject)

		api.GET("tags", tagshandler.GetTags)

		api.POST("/sync", taskHandler.Sync)
	}
	return router
}
