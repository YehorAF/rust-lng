package domain

type Hello struct {
	Message string
}

func (h *Hello) GetHelloMessage() string {
	return h.Message
}

type Message struct {
	Data string `json:"data" xml:"data" form:"data"`
}
