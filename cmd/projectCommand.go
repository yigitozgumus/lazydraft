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
	}
	if commandList[0] == ProjectListSubCommand {
		cp.runProjectListCommand()
	}
	if commandList[0] == ProjectGetActiveSubCommand {
		cp.runProjectGetActiveCommand()
	}
	if commandList[0] == ProjectChangeActiveSubCommand {
		cp.runProjectChangeActiveCommand()
	}
	return nil
}

func (cp *CommandParser) runProjectListCommand() {
	projectList := config.GetProjectList(cp.ProjectConfig)
	projectNames := projectList.GetProjectNames()
	fmt.Println("\nCurrent Project List")
	for index, name := range projectNames {
		fmt.Printf("  %d: %s\n", index+1, name)
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

}
