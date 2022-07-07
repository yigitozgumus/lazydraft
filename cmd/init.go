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
		Usage:   "Create projects.yml to start tracking your projects",
		Action: func(context *cli.Context) error {
			configFile := lazydraft.ConfigFileName
			homeDir, err := os.UserHomeDir()
			if err != nil {
				return err
			}
			configFilePath := homeDir + "/" + configFile
			_, err = os.ReadFile(configFilePath)
			if err == nil {
				fmt.Println("\nprojects.yml file is present")
				return nil
			}
			fmt.Println("\nCreating projects.yml ...")
			configDir := homeDir + "/" + lazydraft.ConfigBaseDir + "/" + lazydraft.ConfigFileDir
			_, err = os.ReadDir(configDir)
			if err != nil {
				os.Mkdir(configDir, 0777)
			}
			ioutil.WriteFile(configFilePath, []byte{}, 0666)
			fmt.Printf("\nprojects.yml is created at '%s'\n", configFilePath)
			return nil
		},
	}
}
