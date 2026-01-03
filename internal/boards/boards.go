package boards

type BoardMeta struct {
	name string
}

type Board interface {
	getMeta() BoardMeta
	auth()
	issues()
}

var Boards = []Board{
	Jira{},
	Clickup{},
}

func GetBoards() []string {
	boards := make([]string, len(Boards))
	for i, board := range Boards {
		boards[i] = board.getMeta().name
	}
	return boards
}
