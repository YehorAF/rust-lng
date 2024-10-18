package ports

import (
	fiber "github.com/gofiber/fiber/v2"
)

type HelloService interface {
}

type HelloRepository interface {
}

type HelloHandlers interface {
	GetHelloMessage(c *fiber.Ctx) error
	SendMessage(c *fiber.Ctx) error
}
