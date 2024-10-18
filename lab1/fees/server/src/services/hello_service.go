package services

import (
	"github.com/YehorAF/feePlatform.git/ports"
)

type HelloService struct {
	helloRepository ports.HelloRepository
}

// використовуємо для перевірки
// var _ ports.HelloService = (*HelloService)(nil)

func NewHelloService(repository ports.HelloRepository) *HelloService {
	return &HelloService{
		helloRepository: repository,
	}
}
