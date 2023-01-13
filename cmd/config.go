package cmd

import (
	"fmt"
	"lazy-publish/lazydraft"
	"lazy-publish/util"
	"os"

	"github.com/urfave/cli/v2"
)

func registerConfigCommand() *cli.Command {
	command := cli.Command{
		Name:    "config",
		Aliases: []string{"c"},
		Usage:   "Configure CLI settings",
		Subcommands: []*cli.Command{
			registerInitCommand(),
			registerResetCommand(),
		},
	}
	return &command
}

func registerInitCommand() *cli.Command {
	return &cli.Command{
		Name:    "init",
		Aliases: []string{"i"},
		Usage:   "Create config files to start tracking your projects",
		Action: func(context *cli.Context) error {
			// create config dir if needed
			homeDir, err := os.UserHomeDir()
			util.CheckErrorAndReturn(err)
			configDir := homeDir + "/" + lazydraft.ConfigBaseDir + "/" + lazydraft.ConfigFileDir
			_, err = os.ReadDir(configDir)
			if err != nil {
				fmt.Println("\nNo lazydraft config directory found. Creating...")
				os.Mkdir(configDir, 0777)
				fmt.Println("\nlazydraft config folder is created.")
			} else {
				fmt.Println("\nlazydraft config directory is present.")
			}
			// create projects file
			util.CreateFileInUserHomeDir(lazydraft.ProjectDataFilePath, "projects.yml")
			// create settings file
			util.CreateFileInUserHomeDir(lazydraft.AppSettingsFilePath, "settings.yml")
			return nil
		},
	}
}

func registerResetCommand() *cli.Command {

	return &cli.Command{
		Name:    "reset",
		Aliases: []string{"r"},
		Usage:   "Delete config folder to start over",
		Action: func(context *cli.Context) error {
			// create config dir if needed
			homeDir, err := os.UserHomeDir()
			util.CheckErrorAndReturn(err)
			configDir := homeDir + "/" + lazydraft.ConfigBaseDir + "/" + lazydraft.ConfigFileDir
			err = os.RemoveAll(configDir)
			if err != nil {
				return err
			}
			fmt.Println("\nConfig folder is removed.")
			return nil
		},
	}
}
