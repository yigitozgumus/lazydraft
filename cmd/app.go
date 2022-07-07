package cmd

import (
	"github.com/urfave/cli/v2"
)

var app = cli.NewApp()

func InitApplication() cli.App {
	initAppInfo()
	registerCommands()
	return *app
}

func registerCommands() {
	app.Commands = []*cli.Command{
		registerProjectCommand(),
		registerDraftCommand(),
	}
}

func initAppInfo() {
	author := cli.Author{
		Name:  "Yigit Ozgumus",
		Email: "yigitozgumus1@gmail.com",
	}
	app.Name = "lazyPublish"
	app.Usage = "Simple application to transfer drafts to your static site"
	app.Authors = []*cli.Author{&author}
	app.Version = "1.0.0"
}
