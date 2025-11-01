package tui

import (
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type SelectorModel struct {
	title    string
	options  []string
	Cursor   int
	IsQuited bool
}

func initialModel(title string, options []string) SelectorModel {
	return SelectorModel{
		title:   title,
		options: options,
	}
}

func (m SelectorModel) Init() tea.Cmd {
	return nil
}

func (m SelectorModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {

	case tea.KeyMsg:

		switch msg.String() {

		case "ctrl+c", "q":
			m.IsQuited = true
			return m, tea.Quit

		case "enter":
			return m, tea.Quit

		case "up", "k":
			if m.Cursor >= 0 {
				m.Cursor--
			}
			if m.Cursor == -1 {
				m.Cursor = len(m.options) - 1
			}

		case "down", "j":
			noOfOptions := len(m.options)
			if m.Cursor < noOfOptions {
				m.Cursor++
			}
			if m.Cursor == noOfOptions {
				m.Cursor = 0
			}
		}
	}

	return m, nil
}

func (m SelectorModel) View() string {

	focusedStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("#FAFAFA")).Padding(0)

	style := lipgloss.NewStyle().Foreground(lipgloss.Color("250")).MarginLeft(0)

	s := m.title

	for i, choice := range m.options {

		cursor := " "
		if m.Cursor == i {
			cursor = ">"

			s += focusedStyle.Render(fmt.Sprintf("\n%s %s", cursor, choice))

		} else {
			s += style.Render(fmt.Sprintf("\n%s %s", cursor, choice))
		}

	}

	s += style.Render("\nPress q to quit.\n")
	return s
}

func ShowSimpleList(title string, options []string) tea.Model {
	p := tea.NewProgram(initialModel(title, options))
	returnModel, err := p.Run()

	if err != nil {
		fmt.Printf("Error when running the program %v", err)
		os.Exit(1)
	}

	return returnModel
}
