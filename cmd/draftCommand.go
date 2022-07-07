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
	fmt.Printf("\n Post drafts of %s\n", activeProject.Name)
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
	fmt.Print("Type post number: ")
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
	fmt.Println("Post is added to the stage.")
	return nil
}

func (cp *CommandParser) runRemoveDraftFromStageCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	stagedDraftPosts, err := activeProject.GetStagedPosts()
	draftIndex, err := cp.getSelectedIndexFromStagedDrafts("Type post number to unstage")
	if err != nil {
		return err
	}
	err = activeProject.RemovePostFromTarget(stagedDraftPosts[draftIndex])
	if err != nil {
		return err
	}
	fmt.Println("\nPost is removed from stage")
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
	fmt.Println("Staged posts are")
	for index, staged := range stagedDraftPosts {
		fmt.Printf(" %d) %s\n", index+1, staged.PostName)
	}
	inputInt, err := config.GetInputFromUser("\n" + inputTestForOperation)
	if inputInt < 1 || inputInt > len(stagedDraftPosts) {
		return -1, errors.New("invalid choice")
	}
	chosenPost := stagedDraftPosts[inputInt-1]
	draftIndex := -1
	for index, post := range activeProject.Posts.PostList {
		if post.PostName == chosenPost.PostName {
			draftIndex = index
			break
		}
	}
	return draftIndex, nil
}

func (cp *CommandParser) runDraftPublishFromStageCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	stagedDraftPosts, err := activeProject.GetStagedPosts()
	draftIndex, err := cp.getSelectedIndexFromStagedDrafts("Type post number to publish")
	if err != nil {
		return err
	}
	postToUpdate := stagedDraftPosts[draftIndex]
	err = activeProject.UpdatePostToLatest(postToUpdate, draftIndex)
	if err != nil {
		return err
	}
	err = activeProject.CopyDraftToPublished(postToUpdate)
	if err != nil {
		return err
	}
	err = activeProject.RemovePostFromDrafts(postToUpdate)
	if err != nil {
		return err
	}
	return nil
}

func (cp *CommandParser) runUpdateStagedDraftCommand() error {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	stagedDraftPosts, err := activeProject.GetStagedPosts()
	draftIndex, err := cp.getSelectedIndexFromStagedDrafts("Type post number to update")
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
	fmt.Println("\nPost is updated")
	return nil
}
