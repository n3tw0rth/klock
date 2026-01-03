package boards

import "fmt"

type Jira struct {
	accessToken string
}

func (Jira) getMeta() BoardMeta {
	return BoardMeta{
		name: "jira",
	}
}

func (Jira) auth() {
	fmt.Println("Jire Auth")
}

func (Jira) issues() {
	fmt.Println("Jira issues")
}
