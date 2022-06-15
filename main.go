package main

import (
	"lazy-publish/config"
)

func main() {
	inputFile := config.InputFile{
		Path: "test-project.yaml",
	}
	project := config.Project{}
	project.InitProject(inputFile)
	project.CopyPostToTarget(0)
}
