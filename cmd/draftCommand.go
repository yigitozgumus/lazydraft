package cmd

import (
	"errors"
	"fmt"
	"lazy-publish/config"
	"strconv"
)

const DraftCommand = "draft"
const DraftListSubCommand = "list"
const DraftCopyToStageSubCommand = "stage"
const DraftUpdateStagedDraftSubCommand = "update-stage"
const DraftRemoveFromStageSubCommand = "unstage"
const DraftPublishFromStageSubCommand = "publish"

func (cp *CommandParser) runDraftCommand(commandList []string) error {
	if len(commandList) == 0 {
		fmt.Println("Draft command usage will be written")
		return nil
	}
	if commandList[0] == DraftListSubCommand {
		err := cp.runDraftListCommand()
		return err
	}
	if commandList[0] == DraftCopyToStageSubCommand {
		err := cp.runDraftCopyToStageCommand()
		return err
	}
	if commandList[0] == DraftUpdateStagedDraftSubCommand {
		err := cp.runUpdateStagedDraftCommand()
		return err
	}
	if commandList[0] == DraftRemoveFromStageSubCommand {
		err := cp.runDraftRemoveFromStageCommand()
		return err
	}
	if commandList[0] == DraftPublishFromStageSubCommand {
		err := cp.runDraftPublishFromStageCommand()
		return err
	}
	return nil
}

func (cp *CommandParser) runDraftListCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	if err != nil {
		return err
	}
	targetFiles, err := activeProject.GetTargetContentDirFiles()
	if err != nil {
		return err
	}
	draftList := activeProject.Posts.PostList
	fmt.Printf("\n Drafts of %s\n", activeProject.Name)
	for index, draft := range draftList {
		stageFound := ""
		for _, targetPost := range targetFiles {
			if targetPost == config.ConvertMarkdownToPostName(draft.PostName) {
				stageFound = "(staged)"
				break
			}
		}
		fmt.Printf("  %d) %s %s\n", index+1, draft.PostName, stageFound)
	}
	return nil
}

func (cp *CommandParser) runDraftCopyToStageCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	if err != nil {
		return err
	}
	cp.runDraftListCommand()
	var input string
	fmt.Print("Type Draft Number: ")
	fmt.Scanln(&input)
	inputInt, err := strconv.Atoi(input)
	if err != nil {
		return err
	}
	if inputInt < 1 || inputInt > len(activeProject.Posts.PostList) {
		return errors.New("invalid choice")
	}
	draftIndex := inputInt - 1
	err = activeProject.CopyPostToTarget(draftIndex)
	if err != nil {
		return err
	}
	fmt.Println("Draft is added to the stage.")
	return nil
}

func (cp *CommandParser) runDraftRemoveFromStageCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	if err != nil {
		return err
	}
	cp.runDraftListCommand()
	activeProject.GetTargetContentDirFiles()
	return nil
}

func (cp *CommandParser) runDraftPublishFromStageCommand() error {
	return nil
}

func (cp *CommandParser) runUpdateStagedDraftCommand() error {
	return nil
}
