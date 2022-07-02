package config

import "errors"

type ProjectList struct {
	projects map[string]Project
}

func GetProjectList(config ProjectConfig) *ProjectList {
	projectList := ProjectList{}
	projectList.initProject(config)
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

func (p *ProjectList) initProject(config ProjectConfig) {
	(*p).projects = make(map[string]Project)
	for projectName, projectData := range config.Data {
		(*p).projects[projectName] = Project{
			Name:   projectName,
			Posts:  projectData.ExtractPostListInfo(),
			Target: projectData.ExtractTargetListInfo(),
			Active: projectData.Active,
		}
	}
}

func (p *ProjectList) GetActiveProject() (Project, error) {
	projects := (*p).projects
	for _, project := range projects {
		if project.Active {
			return project, nil
		}
	}
	return Project{}, errors.New("no active project found")
}
