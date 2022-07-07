package cmd

import (
	"fmt"
	"github.com/urfave/cli/v2"
	"lazy-publish/lazydraft"
)

var app = cli.NewApp()

func InitApplication() cli.App {
	initAppInfo()
	registerAppCommand()
	registerCommands()
	return *app
}

func registerAppCommand() {
	app.Action = func(context *cli.Context) error {
		_, err := lazydraft.GetProjectConfig()
		if err != nil {
			fmt.Println(err.Error())
			return nil
		}
		if context.NArg() == 0 {
			fmt.Println("\n use 'lazydraft help' to see available commands")
		}
		return nil
	}
}

func registerCommands() {
	app.Commands = []*cli.Command{
		registerInitCommand(),
		registerProjectCommand(),
		registerDraftCommand(),
	}
}

func initAppInfo() {
	author := cli.Author{
		Name:  "Yigit Ozgumus",
		Email: "yigitozgumus1@gmail.com",
	}
	app.Name = "lazydraft"
	app.Usage = "Simple application to transfer drafts to your static site"
	app.Authors = []*cli.Author{&author}
	app.Version = "1.0.7"
}
