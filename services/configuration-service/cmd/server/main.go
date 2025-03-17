package main

import (
	"log"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/van1sh/Mirage/services/configuration-service/internal/api"
	"github.com/van1sh/Mirage/services/configuration-service/internal/config"
	"github.com/van1sh/Mirage/services/configuration-service/internal/storage"
)

func main() {
	// Initialize logger
	log.Println("Starting configuration service...")

	// Load application config
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("Failed to load configuration: %v", err)
	}

	// Initialize storage
	store, err := storage.NewFileStorage(cfg.StoragePath)
	if err != nil {
		log.Fatalf("Failed to initialize storage: %v", err)
	}

	// Setup Gin router
	router := gin.Default()

	// Register middleware
	router.Use(gin.Recovery())
	router.Use(api.CORSMiddleware())
	router.Use(api.RequestLogger())

	// Register routes
	api.RegisterRoutes(router, store)

	// Get port from environment or use default
	port := os.Getenv("PORT")
	if port == "" {
		port = "8092"
	}

	// Start the server
	log.Printf("Configuration service listening on port %s", port)
	if err := router.Run(":" + port); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}
