package cmd

import (
	"fmt"
	"github.com/urfave/cli/v2"
	"io/ioutil"
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
			projectDataFile := lazydraft.ProjectDataFilePath
			configFilePath := homeDir + "/" + projectDataFile
			_, err = os.ReadFile(configFilePath)
			if err != nil {
				fmt.Println("\nCreating projects.yml ...")
				ioutil.WriteFile(configFilePath, []byte{}, 0666)
				fmt.Printf("\nprojects.yml is created at '%s'\n", configFilePath)
			} else {
				fmt.Println("\nprojects.yml file is present.")
			}
			// create settings file
			settingsFile := lazydraft.AppSettingsFilePath
			settingsFilePath := homeDir + "/" + settingsFile
			_, err = os.ReadFile(settingsFilePath)
			if err != nil {
				fmt.Println("\nCreating settings.yml ...")
				ioutil.WriteFile(settingsFilePath, []byte{}, 0666)
				fmt.Printf("\nsettings.yml is createt at '%s'\n", settingsFilePath)
			} else {
				fmt.Println("\nsettings.yml file is present.")
			}
			return nil
		},
	}
}
