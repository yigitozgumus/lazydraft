package lazydraft

import (
	"errors"
	"lazy-publish/util"
)

type ProjectList struct {
	projects map[string]Project
}

func InitProjectList() *ProjectList {
	config, err := getProjectListData()
	util.HandleError(err)
	projectList := ProjectList{}
	projectList.init(*config)
	return &projectList
}

func (p *ProjectList) GetProjectNames() []string {
	keys := make([]string, 0, len((*p).projects))
	for key, _ := range (*p).projects {
		keys = append(keys, key)
	}
	return keys
}

func (p *ProjectList) GetProjectDataOf(name string) Project {
	return (*p).projects[name]
}

func (p *ProjectList) init(config ProjectPathList) {
	(*p).projects = make(map[string]Project)
	for projectName, projectData := range config.Data {
		(*p).projects[projectName] = Project{
			Name:         projectName,
			Posts:        projectData.ExtractPostListInfo(),
			PublishedDir: projectData.ExtractTargetPublishedDir(),
			Target:       projectData.ExtractTargetListInfo(),
		}
	}
}

func (p *ProjectList) GetActiveProject(settings *AppSettings) (Project, error) {
	projects := (*p).projects
	activeProject := settings.ActiveProject
	for _, project := range projects {
		if project.Name == activeProject {
			return project, nil
		}
	}
	return Project{}, errors.New("\nno active project found. See 'lazydraft project config'")
}
