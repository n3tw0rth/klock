package boards

import "fmt"

type Jira struct {
	accessToken string
}

func (j Jira) auth() {
	fmt.Println("Jire Auth")
}

func (j Jira) issues() {
	fmt.Println("Jira issues")
}
