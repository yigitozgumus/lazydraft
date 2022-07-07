package cmd

import (
	"errors"
	"fmt"
	"github.com/urfave/cli/v2"
	"lazy-publish/lazypublish"
	"strconv"
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
	pc, err := lazypublish.GetProjectConfig()
	if err != nil {
		return err
	}
	projectList := lazypublish.GetProjectList(*pc)
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
			if targetPost == lazypublish.ConvertMarkdownToPostName(draft.PostName) {
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
			pc, err := lazypublish.GetProjectConfig()
			if err != nil {
				return err
			}
			projectList := lazypublish.GetProjectList(*pc)
			activeProject, err := projectList.GetActiveProject()
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
			pc, err := lazypublish.GetProjectConfig()
			if err != nil {
				return err
			}
			projectList := lazypublish.GetProjectList(*pc)
			activeProject, err := projectList.GetActiveProject()
			stagedDraftPosts, err := activeProject.GetStagedPosts()
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

func getSelectedIndexFromStagedDrafts(pc lazypublish.ProjectConfig, inputTestForOperation string) (int, error) {
	projectList := lazypublish.GetProjectList(pc)
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
	inputInt, err := lazypublish.GetInputFromUser("\n" + inputTestForOperation)
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
		Name:    "unstage",
		Aliases: []string{"u"},
		Usage:   "remove the staged draft",
		Action: func(context *cli.Context) error {
			pc, err := lazypublish.GetProjectConfig()
			if err != nil {
				return err
			}
			projectList := lazypublish.GetProjectList(*pc)
			activeProject, err := projectList.GetActiveProject()
			stagedDraftPosts, err := activeProject.GetStagedPosts()
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
			pc, err := lazypublish.GetProjectConfig()
			if err != nil {
				return err
			}
			projectList := lazypublish.GetProjectList(*pc)
			activeProject, err := projectList.GetActiveProject()
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
