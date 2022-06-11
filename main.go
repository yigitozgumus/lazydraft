package main

import (
	"fmt"
	"lazy-publish/config"
)

func main() {
	inputFile := config.InputFile{
		Path: "test-project.yaml",
	}
	project := config.Project{}
	project.InitProject(inputFile)
	fmt.Println(project.Posts)
}
