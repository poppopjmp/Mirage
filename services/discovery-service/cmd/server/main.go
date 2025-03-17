package main

import (
	"log"
	"os"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/van1sh/Mirage/services/discovery-service/internal/api"
	"github.com/van1sh/Mirage/services/discovery-service/internal/config"
	"github.com/van1sh/Mirage/services/discovery-service/internal/registry"
)

func main() {
	// Initialize logger
	log.Println("Starting discovery service...")

	// Load application config
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("Failed to load configuration: %v", err)
	}

	// Initialize service registry
	reg := registry.NewInMemoryRegistry(time.Duration(cfg.HeartbeatTimeoutSec) * time.Second)

	// Setup periodic cleanup of stale services
	go func() {
		ticker := time.NewTicker(30 * time.Second)
		defer ticker.Stop()

		for range ticker.C {
			reg.CleanupStaleServices()
		}
	}()

	// Setup Gin router
	router := gin.Default()

	// Register middleware
	router.Use(gin.Recovery())
	router.Use(api.CORSMiddleware())
	router.Use(api.RequestLogger())

	// Register routes
	api.RegisterRoutes(router, reg)

	// Get port from environment or use default
	port := os.Getenv("PORT")
	if port == "" {
		port = "8093"
	}

	// Start the server
	log.Printf("Discovery service listening on port %s", port)
	if err := router.Run(":" + port); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}
