package config

import (
	"gopkg.in/yaml.v3"
	"io/ioutil"
	"log"
)

type InputFile struct {
	Path string
}

type YamlHandler interface {
	ReadYamlFile() YamlFile
}

func (yf InputFile) ReadYamlFile() YamlFile {
	data, err := ioutil.ReadFile(yf.Path)
	if err != nil {
		log.Fatalf("error: %v", err)
	}

	config := YamlFile{}
	err = yaml.Unmarshal(data, &config)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	return config
}
