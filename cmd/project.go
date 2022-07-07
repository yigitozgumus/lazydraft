package cmd

import (
	"fmt"
	"github.com/urfave/cli/v2"
	"lazy-publish/lazypublish"
	"log"
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
			pc, err := lazypublish.GetProjectConfig()
			if err != nil {
				return err
			}
			projectList := lazypublish.GetProjectList(*pc)
			projectNames := projectList.GetProjectNames()
			activeProject, _ := projectList.GetActiveProject()
			activeProjectName := activeProject.Name
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
			pc, err := lazypublish.GetProjectConfig()
			if err != nil {
				return err
			}
			projectList := lazypublish.GetProjectList(*pc)
			activeProject, err := projectList.GetActiveProject()
			if err != nil {
				log.Fatal(err)
			}
			fmt.Printf("\nCurrent active project is %s\n", activeProject.Name)
			return nil
		},
	}
}

func registerChangeActiveProjectCommand() *cli.Command {
	return &cli.Command{
		Name:    "change",
		Aliases: []string{"c"},
		Usage:   "Change your active project for draft management",
		Action: func(context *cli.Context) error {
			fmt.Println("will be implemented")
			return nil
		},
	}
}
