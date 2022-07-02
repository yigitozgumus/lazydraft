package cmd

import (
	"fmt"
	"lazy-publish/config"
	"log"
)

const ProjectCommand = "project"
const ProjectListSubCommand = "list"
const ProjectGetActiveSubCommand = "active"
const ProjectChangeActiveSubCommand = "change-active"

func (cp *CommandParser) runProjectCommand(commandList []string) error {
	if len(commandList) == 0 {
		fmt.Println("Project command usage will be written")
		return nil
	}
	if commandList[0] == ProjectListSubCommand {
		cp.runProjectListCommand()
		return nil
	}
	if commandList[0] == ProjectGetActiveSubCommand {
		cp.runProjectGetActiveCommand()
		return nil
	}
	if commandList[0] == ProjectChangeActiveSubCommand {
		cp.runProjectChangeActiveCommand()
		return nil
	}
	return nil
}

func (cp *CommandParser) runProjectListCommand() {
	projectList := config.GetProjectList(cp.ProjectConfig)
	projectNames := projectList.GetProjectNames()
	activeProject, _ := projectList.GetActiveProject()
	activeProjectName := activeProject.Name
	fmt.Println("\nCurrent Project List")
	for index, name := range projectNames {
		activeOutput := ""
		if name == activeProjectName {
			activeOutput = "(active)"
		}
		fmt.Printf("  %d) %s %s\n", index+1, name, activeOutput)
	}
}

func (cp *CommandParser) runProjectGetActiveCommand() {
	projectList := config.GetProjectList(cp.ProjectConfig)
	activeProject, err := projectList.GetActiveProject()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("\nCurrent active project is %s\n", activeProject.Name)
}

func (cp *CommandParser) runProjectChangeActiveCommand() {
	cp.runProjectListCommand()
	var input string
	fmt.Println("Type project number")
	fmt.Scanln(&input)
}
