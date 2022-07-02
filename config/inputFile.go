package config

import (
	"gopkg.in/yaml.v3"
	"io/ioutil"
	"log"
)

type InputFile struct {
	Path string
}

func (yf InputFile) ReadYamlFile() map[string]YamlFile {
	data, err := ioutil.ReadFile(yf.Path)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	config := make(map[string]YamlFile)
	err = yaml.Unmarshal(data, &config)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	return config
}
