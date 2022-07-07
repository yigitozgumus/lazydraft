package main

import (
	"fmt"
	"lazy-publish/cmd"
	"lazy-publish/config"
	"log"
	"os"
)

func main() {
	projectConfig, err := config.GetProjectConfig()
	if err != nil {
		log.Fatal(err)
	}
	if len(os.Args) < 2 {
		fmt.Println("Usage will be written")
		return
	}
	parser := cmd.InitCommandParser(*projectConfig)
	err = parser.ParseCommands(os.Args[1:])
	if err != nil {
		log.Fatal(err)
	}
}
