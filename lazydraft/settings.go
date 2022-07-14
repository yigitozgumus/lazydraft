package lazydraft

import (
	"errors"
	"gopkg.in/yaml.v3"
	"io/ioutil"
	"os"
)

const AppSettingsFilePath = ".config/lazydraft/settings.yml"

type AppSettings struct {
	ActiveProject string `yaml:"activeProject"`
}

func GetSettings() (*AppSettings, error) {
	settingsPath, err := getSettingsPath()
	if err != nil {
		return nil, errors.New(settingsFilePathError)
	}
	appSettings, err := settingsPath.readSettings()
	if err != nil {
		return nil, err
	}
	return appSettings, nil
}

func getSettingsPath() (*InputFile, error) {
	dirName, err := os.UserHomeDir()
	if err != nil {
		return nil, errors.New(userHomeDirectoryError)
	}
	settingsFile := InputFile{
		Path: dirName + "/" + AppSettingsFilePath,
	}
	return &settingsFile, nil
}

func (yf InputFile) readSettings() (*AppSettings, error) {
	data, err := ioutil.ReadFile(yf.Path)
	if err != nil {
		return nil, errors.New(settingsFileDoesNotExistsError)
	}
	settings := AppSettings{}
	err = yaml.Unmarshal(data, &settings)
	if err != nil {
		return nil, errors.New(settingsFormatInvalidError)
	}
	return &settings, nil
}
