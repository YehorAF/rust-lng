package repositories

// import (
// 	"github.com/YehorAF/feePlatform.git/ports"
// )

type HelloRepository struct {
}

// використовуємо для перевірки
// var _ ports.HelloRepository = (*HelloRepository)(nil)

func NewHelloRepository() *HelloRepository {
	return &HelloRepository{}
}
