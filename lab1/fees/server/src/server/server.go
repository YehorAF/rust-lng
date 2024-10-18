package server

import (
	"log"

	"github.com/YehorAF/feePlatform.git/ports"
	fiber "github.com/gofiber/fiber/v2"
)

type Server struct {
	helloHandlers ports.HelloHandlers
}

func NewServer(helloHandlers ports.HelloHandlers) *Server {
	return &Server{
		helloHandlers: helloHandlers,
	}
}

func (s *Server) Initialize() {
	app := fiber.New()
	v1 := app.Group("/")

	helloRoutes := v1.Group("/")
	helloRoutes.Get("/", s.helloHandlers.GetHelloMessage)
	helloRoutes.Post("/", s.helloHandlers.SendMessage)

	err := app.Listen(":8000")
	if err != nil {
		log.Fatal(err)
	}
}
