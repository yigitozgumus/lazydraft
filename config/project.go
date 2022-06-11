package config

type Post struct {
	FilePath string
	Name     string
}

type Project struct {
	Base  string
	Posts []Post
	//TargetBase    string
	//TargetContent string
	//TargetAsset   string
}

type projectOperations interface {
	InitProject(f InputFile)
}

func (p *Project) InitProject(f InputFile) {
	yamlFile := f.ReadYamlFile()
	*p = yamlFile.ConvertYamlToProject()
}
