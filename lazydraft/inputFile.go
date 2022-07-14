package lazydraft

import (
	"errors"
	"gopkg.in/yaml.v3"
	"io/ioutil"
	"os"
)

const projectFileDoesNotExistsError = "\nprojects.yml file could not be found. See 'lazydraft help init'"
const settingsFileDoesNotExistsError = "\nsettings.yml file could not be found. See 'lazydraft help init'"
const projectFormatInvalidError = "\nprojects.yml file format is invalid. See 'lazydraft config'"
const settingsFormatInvalidError = "\nsettings.yml file format is invalid. See 'lazydraft config'"
const ProjectDataFilePath = ".config/lazydraft/projects.yml"
const ConfigFileDir = "lazydraft"
const ConfigBaseDir = ".config"
const userHomeDirectoryError = "user home directory cannot be retrieved"
const projectDataFilePathError = "projects.yml file path cannot be retrieved"
const settingsFilePathError = "settings.yml file path cannot be retrieved"

type InputFile struct {
	Path string
}

type ProjectPathList struct {
	Data map[string]YamlFile
}

func (yf InputFile) readProjectPathList() (*ProjectPathList, error) {
	data, err := ioutil.ReadFile(yf.Path)
	if err != nil {
		return nil, errors.New(projectFileDoesNotExistsError)
	}
	projectConfig := ProjectPathList{
		Data: make(map[string]YamlFile),
	}
	err = yaml.Unmarshal(data, &projectConfig.Data)
	if err != nil {
		return nil, errors.New(projectFormatInvalidError)
	}
	return &projectConfig, nil
}

func GetProjectListData() (*ProjectPathList, error) {
	projectListPath, err := getProjectPathDataPath()
	if err != nil {
		return nil, errors.New(projectDataFilePathError)
	}
	projectConfig, err := projectListPath.readProjectPathList()
	if err != nil {
		return nil, err
	}
	return projectConfig, nil
}

func getProjectPathDataPath() (*InputFile, error) {
	dirname, err := os.UserHomeDir()
	if err != nil {
		return nil, errors.New(userHomeDirectoryError)
	}
	configFile := InputFile{
		Path: dirname + "/" + ProjectDataFilePath,
	}
	return &configFile, nil
}
