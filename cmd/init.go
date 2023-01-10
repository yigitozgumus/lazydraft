package cmd

import (
	"fmt"
	"github.com/urfave/cli/v2"
	"lazy-publish/lazydraft"
	"os"
)

func registerInitCommand() *cli.Command {
	return &cli.Command{
		Name:    "init",
		Aliases: []string{"i"},
		Usage:   "Create config files to start tracking your projects",
		Action: func(context *cli.Context) error {
			// create config dir if needed
			homeDir, err := os.UserHomeDir()
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
			lazydraft.CreateFileInUserHomeDir(lazydraft.ProjectDataFilePath, "projects.yml")
			// create settings file
			lazydraft.CreateFileInUserHomeDir(lazydraft.AppSettingsFilePath, "settings.yml")
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
