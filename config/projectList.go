package config

import "errors"

type ProjectList struct {
	projects map[string]Project
}

func (p *ProjectList) getProjectNames() []string {
	keys := make([]string, 0, len((*p).projects))
	for key, _ := range (*p).projects {
		keys = append(keys, key)
	}
	return keys
}

func (p *ProjectList) getProjectDataOf(name string) Project {
	return (*p).projects[name]
}

func (p *ProjectList) InitProject(f InputFile) {
	yamlFile := f.ReadYamlFile()
	(*p).projects = make(map[string]Project)
	for projectName, projectData := range yamlFile {
		(*p).projects[projectName] = Project{
			Name:   projectName,
			Posts:  projectData.ExtractPostListInfo(),
			Target: projectData.ExtractTargetListInfo(),
			Active: projectData.Active,
		}
	}
}

func (p *ProjectList) getActiveProject() (Project, error) {
	projects := (*p).projects
	for _, project := range projects {
		if project.Active {
			return project, nil
		}
	}
	return Project{}, errors.New("no active project found")
}
