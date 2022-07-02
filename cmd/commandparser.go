package cmd

import (
	"lazy-publish/config"
)
import s "strings"

type CommandParser struct {
	ProjectConfig config.ProjectConfig
}

func InitCommandParser(projectConfig config.ProjectConfig) *CommandParser {
	parser := CommandParser{
		ProjectConfig: projectConfig,
	}
	return &parser
}

func (cp *CommandParser) ParseCommands(commandList []string) error {
	commandToParse := commandList[0]
	if s.HasPrefix(commandToParse, "--") {
		err := cp.runRootCommand()
		return err
	}
	if commandToParse == DraftCommand {
		err := cp.runDraftCommand(commandList[1:])
		return err
	}
	if commandToParse == ProjectCommand {
		err := cp.runProjectCommand(commandList[1:])
		return err
	}
	return nil
}
