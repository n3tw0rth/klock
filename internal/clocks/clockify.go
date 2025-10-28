package clocks

import "fmt"

type Clockify struct {
	accessToken string
}

func (c Clockify) auth() {
	fmt.Println("Clockify Auth")
}

func (c Clockify) log() {
	fmt.Println("Clockify log")
}
