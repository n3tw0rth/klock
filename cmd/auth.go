package cmd

import (
	"fmt"

	"github.com/n3tw0rth/jired/internal/tui"
	"github.com/spf13/cobra"
)

var authCmd = &cobra.Command{
	Use:   "auth",
	Short: "A brief description of your command",
	Args:  cobra.ExactArgs(1),
}

var loginCmd = &cobra.Command{
	Use:   "login",
	Short: "A brief description of your command",
	Run: func(cmd *cobra.Command, args []string) {
		options := []string{"Board", "Clock"}
		selectedItem := tui.ShowSimpleList("Select the Integration Type:", options)

		result := selectedItem.(tui.SelectorModel)

		println("selected item %s", result.Cursor)
	},
}

var logoutCmd = &cobra.Command{
	Use:   "logout",
	Short: "A brief description of your command",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("login called")
	},
}

func init() {
	rootCmd.AddCommand(authCmd)
	authCmd.AddCommand(loginCmd, logoutCmd)
}
