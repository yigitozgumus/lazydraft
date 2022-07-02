package cmd

import "fmt"

const DraftCommand = "draft"
const DraftListSubCommand = "list"

func (cp *CommandParser) runDraftCommand(commandList []string) error {
	fmt.Println("draft command not implemented yet")
	return nil
}
