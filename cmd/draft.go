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
			registerListDraftCommand(),
			registerCopyDraftToStageCommand(),
			registerUpdateStagedDraftCommand(),
			registerRemoveDraftFromStageCommand(),
			registerPublishDraftCommand(),
		},
	}
	return &command
}

func registerListDraftCommand() *cli.Command {
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
	util.CheckErrorAndReturn(err)
	projectList := lazydraft.GetProjectList(*pc)
	settings, err := lazydraft.GetSettings()
	util.CheckErrorAndReturn(err)
	activeProject, err := projectList.GetActiveProject(settings)
	util.CheckErrorAndReturn(err)
	targetFiles, err := activeProject.GetTargetContentDirFiles()
	util.CheckErrorAndReturn(err)
	draftList := activeProject.Posts.PostList
	fmt.Printf("\n Post drafts of %s\n", activeProject.Name)
	for index, draft := range draftList {
		stageFound := ""
		for _, targetPost := range targetFiles {
			if targetPost == lazydraft.ConvertMarkdownToPostName(draft.PostName) {
				stageFound = "(staged)"
				break
			}
		}
		fmt.Printf("  %d) %s %s\n", index+1, draft.PostName, stageFound)
	}
	return nil
}

func registerCopyDraftToStageCommand() *cli.Command {
	return &cli.Command{
		Name:    "stage",
		Aliases: []string{"s"},
		Usage:   "adds your draft to staging area",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			if err != nil {
				return err
			}
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			if err != nil {
				return err
			}
			activeProject, err := projectList.GetActiveProject(settings)
			if err != nil {
				return err
			}
			runListCommand()
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
		},
	}
}

func registerUpdateStagedDraftCommand() *cli.Command {
	return &cli.Command{
		Name:    "update",
		Aliases: []string{"u"},
		Usage:   "updates the version of the draft in the stage",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			if err != nil {
				return err
			}
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			if err != nil {
				return err
			}
			activeProject, err := projectList.GetActiveProject(settings)
			if err != nil {
				return err
			}
			stagedDraftPosts := activeProject.Posts.PostList
			draftIndex, err := getSelectedIndexFromStagedDrafts(*pc, "Type post number to update")
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
	inputInt, err := lazydraft.GetInputFromUser("\n" + inputTestForOperation)
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

func registerRemoveDraftFromStageCommand() *cli.Command {
	return &cli.Command{
		Name:    "remove",
		Aliases: []string{"r"},
		Usage:   "remove the staged draft",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			if err != nil {
				return err
			}
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			if err != nil {
				return err
			}
			activeProject, err := projectList.GetActiveProject(settings)
			stagedDraftPosts := activeProject.Posts.PostList
			draftIndex, err := getSelectedIndexFromStagedDrafts(*pc, "Type post number to unstage")
			if err != nil {
				return err
			}
			err = activeProject.RemovePostFromTarget(stagedDraftPosts[draftIndex])
			if err != nil {
				return err
			}
			fmt.Println("\nPost is removed from stage")
			return nil
		},
	}
}

func registerPublishDraftCommand() *cli.Command {
	return &cli.Command{
		Name:    "publish",
		Aliases: []string{"p"},
		Usage:   "updates the draft to the latest version and publishes it",
		Action: func(context *cli.Context) error {
			pc, err := lazydraft.GetProjectListData()
			if err != nil {
				return err
			}
			projectList := lazydraft.GetProjectList(*pc)
			settings, err := lazydraft.GetSettings()
			if err != nil {
				return err
			}
			activeProject, err := projectList.GetActiveProject(settings)
			stagedDraftPosts, err := activeProject.GetStagedPosts()
			draftIndex, err := getSelectedIndexFromStagedDrafts(*pc, "Type post number to publish")
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
		},
	}
}
