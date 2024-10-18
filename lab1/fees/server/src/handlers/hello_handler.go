package handlers

import (
	"fmt"

	"github.com/YehorAF/feePlatform.git/domain"
	"github.com/YehorAF/feePlatform.git/ports"

	fiber "github.com/gofiber/fiber/v2"
)

type HelloHandlers struct {
	helloService ports.HelloService
}

// використовуємо для перевірки
// var _ ports.HelloHandlers = (*HelloHandlers)(nil)

func NewHelloHandlers(helloService ports.HelloService) *HelloHandlers {
	return &HelloHandlers{
		helloService: helloService,
	}
}

func (h *HelloHandlers) GetHelloMessage(c *fiber.Ctx) error {
	if err := c.JSON(domain.Hello{Message: "Hello World"}); err != nil {
		return err
	}
	return nil
}

func (h *HelloHandlers) SendMessage(c *fiber.Ctx) error {
	message := new(domain.Message)

	if err := c.BodyParser(message); err != nil {
		return err
	}

	message.Data = fmt.Sprintf("answer from sever: %s", message.Data)

	if err := c.JSON(message); err != nil {
		return err
	}

	return nil
}
