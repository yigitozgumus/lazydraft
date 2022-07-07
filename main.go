package main

import (
	"lazy-publish/cmd"
	"log"
	"os"
)

func main() {
	app := cmd.InitApplication()
	err := app.Run(os.Args)
	if err != nil {
		log.Fatal(err)
	}
}
