package boards

import "fmt"

type Clickup struct {
	accessToken string
}

func (Clickup) getMeta() BoardMeta {
	return BoardMeta{
		name: "clickup",
	}
}

func (Clickup) auth() {
	fmt.Println("Clickup Auth")
}

func (Clickup) issues() {
	fmt.Println("Clickup issues")
}
