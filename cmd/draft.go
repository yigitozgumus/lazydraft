package cmd

import (
	"errors"
	"fmt"
	"lazy-publish/lazydraft"
	"lazy-publish/util"
	"strconv"

	"github.com/urfave/cli/v2"
)

func registerDraftCommand() *cli.Command {
	command := cli.Command{
		Name:    "draft",
		Aliases: []string{"d"},
		Usage:   "Manage your drafts in your project",
		Subcommands: []*cli.Command{
			registerDraftListCommand(),
			registerDraftStageCommand(),
			registerDraftUpdateCommand(),
			registerDraftRemoveCommand(),
			registerDraftPublishCommand(),
		},
	}
	return &command
}

func registerDraftListCommand() *cli.Command {
	return &cli.Command{
		Name:    "list",
		Aliases: []string{"l"},
		Usage:   "lists all your drafts",
		Action: func(context *cli.Context) error {
			return runListCommand()
		},
	}
}

func runListCommand() error {
	pc, err := lazydraft.GetProjectListData()
	util.HandleError(err)
	projectList := lazydraft.GetProjectList(*pc)
	settings, err := lazydraft.GetSettings()
	util.HandleError(err)
	activeProject, err := projectList.GetActiveProject(settings)
	util.HandleError(err)
	targetFiles, err := activeProject.GetTargetContentDirFiles()
	util.HandleError(err)
	draftList := activeProject.Posts.PostList
	fmt.Printf("\n Post drafts of %s\n", activeProject.Name)
	for index, draft := range draftList {
		stageFound := ""
		for _, targetPost := range targetFiles {
			if targetPost == util.ConvertMarkdownToPostName(draft.PostName) {
				stageFound = "(staged)"
				break
			}
		}
		fmt.Printf("  %d) %s %s\n", index+1, draft.PostName, stageFound)
	}
	return nil
}

func registerDraftStageCommand() *cli.Command {
	return &cli.Command{
		Name:    "stage",
		Aliases: []string{"s"},
		Usage:   "adds your draft to staging area",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			util.HandleError(err)
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			util.HandleError(err)
			activeProject, err := projectList.GetActiveProject(settings)
			util.HandleError(err)
			runListCommand()
			var input string
			fmt.Print("Type post number: ")
			fmt.Scanln(&input)
			inputInt, err := strconv.Atoi(input)
			util.HandleError(err)
			if inputInt < 1 || inputInt > len(activeProject.Posts.PostList) {
				return errors.New("invalid choice")
			}
			draftIndex := inputInt - 1
			err = activeProject.CopyPostToTarget(draftIndex)
			util.HandleError(err)
			fmt.Println("Post is added to the stage.")
			return nil
		},
	}
}

func registerDraftUpdateCommand() *cli.Command {
	return &cli.Command{
		Name:    "update",
		Aliases: []string{"u"},
		Usage:   "updates the version of the draft in the stage",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			util.HandleError(err)
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			util.HandleError(err)
			activeProject, err := projectList.GetActiveProject(settings)
			util.HandleError(err)
			stagedDraftPosts := activeProject.Posts.PostList
			draftIndex, err := getSelectedIndexFromStagedDrafts(*pc, "Type post number to update")
			util.HandleError(err)
			err = activeProject.RemovePostFromTarget(stagedDraftPosts[draftIndex])
			util.HandleError(err)
			err = activeProject.CopyPostToTarget(draftIndex)
			util.HandleError(err)
			fmt.Println("\nPost is updated")
			return nil
		},
	}
}

func getSelectedIndexFromStagedDrafts(pc lazydraft.ProjectPathList, inputTestForOperation string) (int, error) {
	projectList := lazydraft.GetProjectList(pc)
	settings, err := lazydraft.GetSettings()
	if err != nil {
		return -1, err
	}
	activeProject, err := projectList.GetActiveProject(settings)
	if err != nil {
		return -1, err
	}
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
	inputInt, err := util.GetInputFromUser("\n" + inputTestForOperation)
	if err != nil {
		return -1, err
	}
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

func registerDraftRemoveCommand() *cli.Command {
	return &cli.Command{
		Name:    "remove",
		Aliases: []string{"r"},
		Usage:   "remove the staged draft",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			util.HandleError(err)
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			util.HandleError(err)
			activeProject, err := projectList.GetActiveProject(settings)
			util.HandleError(err)
			stagedDraftPosts := activeProject.Posts.PostList
			draftIndex, err := getSelectedIndexFromStagedDrafts(*pc, "Type post number to unstage")
			util.HandleError(err)
			err = activeProject.RemovePostFromTarget(stagedDraftPosts[draftIndex])
			util.HandleError(err)
			fmt.Println("\nPost is removed from stage")
			return nil
		},
	}
}

func registerDraftPublishCommand() *cli.Command {
	return &cli.Command{
		Name:    "publish",
		Aliases: []string{"p"},
		Usage:   "updates the draft to the latest version and publishes it",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			util.HandleError(err)
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			util.HandleError(err)
			activeProject, err := projectList.GetActiveProject(settings)
			util.HandleError(err)
			stagedDraftPosts, err := activeProject.GetStagedPosts()
			util.HandleError(err)
			draftIndex, err := getSelectedIndexFromStagedDrafts(*pc, "Type post number to publish")
			util.HandleError(err)
			postToUpdate := stagedDraftPosts[draftIndex]
			err = activeProject.UpdatePostToLatest(postToUpdate, draftIndex)
			util.HandleError(err)
			err = activeProject.CopyDraftToPublished(postToUpdate)
			util.HandleError(err)
			err = activeProject.RemovePostFromDrafts(postToUpdate)
			util.HandleError(err)
			return nil
		},
	}
}
