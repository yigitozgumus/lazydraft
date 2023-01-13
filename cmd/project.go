package cmd

import (
	"fmt"
	"lazy-publish/lazydraft"
	"lazy-publish/util"

	"github.com/urfave/cli/v2"
)

func registerProjectCommand() *cli.Command {
	command := cli.Command{
		Name:    "project",
		Aliases: []string{"p"},
		Usage:   "Track your projects",
		Subcommands: []*cli.Command{
			registerProjectListCommand(),
			registerGetActiveProjectCommand(),
			registerChangeActiveProjectCommand(),
		},
	}
	return &command
}

func registerProjectListCommand() *cli.Command {
	return &cli.Command{
		Name:    "list",
		Aliases: []string{"l"},
		Usage:   "List your current projects",
		Action: func(context *cli.Context) error {
			settings, err := lazydraft.GetSettings()
			util.HandleError(err)
			if settings.ActiveProject == "" {
				fmt.Println("\nNo Active project found. see 'lazydraft project config'")
				return nil
			}
			projectList := lazydraft.InitProjectList()
			projectNames := projectList.GetProjectNames()
			activeProjectName := settings.ActiveProject
			fmt.Println("\nCurrent Project List")
			for index, name := range projectNames {
				activeOutput := ""
				if name == activeProjectName {
					activeOutput = "(active)"
				}
				fmt.Printf("  %d) %s %s\n", index+1, name, activeOutput)
			}
			return nil
		},
	}
}

func registerGetActiveProjectCommand() *cli.Command {
	return &cli.Command{
		Name:    "active",
		Aliases: []string{"a"},
		Usage:   "Get your active project for draft management",
		Action: func(context *cli.Context) error {
			settings, err := lazydraft.GetSettings()
			util.HandleError(err)
			projectList := lazydraft.InitProjectList()
			activeProject, err := projectList.GetActiveProject(settings)
			util.HandleError(err)
			fmt.Printf("\nCurrent active project is %s\n", activeProject.Name)
			return nil
		},
	}
}

func registerChangeActiveProjectCommand() *cli.Command {
	return &cli.Command{
		Name:    "config",
		Aliases: []string{"c"},
		Usage:   "Change your active project for draft management",
		Action: func(context *cli.Context) error {
			settings, err := lazydraft.GetSettings()
			util.HandleError(err)
			projectList := lazydraft.InitProjectList()
			projectNames := projectList.GetProjectNames()
			fmt.Println("\nCurrent Project List")
			for index, name := range projectNames {
				fmt.Printf("  %d) %s\n", index+1, name)
			}
			projectOrder, err := util.GetInputFromUser("\n Select project to make it active")
			util.HandleError(err)
			if projectOrder < 1 || projectOrder > len(projectNames) {
				fmt.Println("\nInvalid post selection")
				return nil
			}
			projectIndex := projectOrder - 1
			settings.ActiveProject = projectNames[projectIndex]
			lazydraft.UpdateSettings(settings)
			return nil
		},
	}
}
