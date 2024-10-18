package main

import (
	"github.com/YehorAF/feePlatform.git/handlers"
	"github.com/YehorAF/feePlatform.git/repositories"
	"github.com/YehorAF/feePlatform.git/server"
	"github.com/YehorAF/feePlatform.git/services"
)

func main() {
	helloRepository := repositories.NewHelloRepository()
	helloService := services.NewHelloService(helloRepository)
	helloHandlers := handlers.NewHelloHandlers(helloService)
	httpServer := server.NewServer(helloHandlers)
	httpServer.Initialize()
}
