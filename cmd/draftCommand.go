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
const DraftUpdateStagedDraftSubCommand = "update"
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
		err := cp.runRemoveDraftFromStageCommand()
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

func (cp *CommandParser) runRemoveDraftFromStageCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	stagedDraftPosts, err := activeProject.GetStagedPosts()
	draftIndex, err := cp.getSelectedIndexFromStagedDrafts("Type draft number to delete")
	if err != nil {
		return err
	}
	err = activeProject.RemovePostFromTarget(stagedDraftPosts[draftIndex])
	if err != nil {
		return err
	}
	fmt.Println("\nDraft is removed from stage")
	return nil
}

func (cp *CommandParser) getSelectedIndexFromStagedDrafts(inputTestForOperation string) (int, error) {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	stagedDraftPosts, err := activeProject.GetStagedPosts()
	if err != nil {
		return -1, err
	}
	if len(stagedDraftPosts) == 0 {
		return -1, errors.New("there are no staged drafts")
	}
	fmt.Println("Staged drafts are")
	for index, staged := range stagedDraftPosts {
		fmt.Printf(" %d) %s\n", index+1, staged.PostName)
	}
	inputInt, err := config.GetInputFromUser("\n" + inputTestForOperation)
	if inputInt < 1 || inputInt > len(stagedDraftPosts) {
		return -1, errors.New("invalid choice")
	}
	draftIndex := inputInt - 1
	return draftIndex, nil
}

func (cp *CommandParser) runDraftPublishFromStageCommand() error {
	return nil
}

func (cp *CommandParser) runUpdateStagedDraftCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	stagedDraftPosts, err := activeProject.GetStagedPosts()
	draftIndex, err := cp.getSelectedIndexFromStagedDrafts("Type draft number to update")
	if err != nil {
		return err
	}
	err = activeProject.RemovePostFromTarget(stagedDraftPosts[draftIndex])
	if err != nil {
		return err
	}
	err = activeProject.CopyPostToTarget(draftIndex)
	if err != nil {
		return err
	}
	fmt.Println("\nDraft is updated")
	return nil
}
