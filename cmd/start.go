package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

var startCmd = &cobra.Command{
	Use:   "start",
	Short: "Record start working on a issue",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("start called")
	},
}

func init() {
	rootCmd.AddCommand(startCmd)
}
