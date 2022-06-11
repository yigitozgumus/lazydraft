package config

import (
	"io/ioutil"
	"log"
	s "strings"
)

type YamlFile struct {
	Source struct {
		SourceDir string `yaml:"source_dir"`
		PostDir   string `yaml:"post_dir"`
	}
	Target struct {
		BaseDir    string `yaml:"base_dir"`
		ContentDir string `json:"content_dir,omitempty"`
		AssetDir   string `json:"asset_dir,omitempty"`
	}
}

type converter interface {
	ConvertYamlToProject() Project
}

func (yf YamlFile) ConvertYamlToProject() Project {
	base := yf.Source.SourceDir
	postsDir := yf.Source.SourceDir + "/" + yf.Source.PostDir
	posts, err := ioutil.ReadDir(postsDir)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	postPathList := make([]Post, len(posts))
	for index, post := range posts {
		postPathList[index] = Post{
			Name:     convertPostName(post.Name()),
			FilePath: postsDir + "/" + post.Name(),
		}
	}
	return Project{
		Base:  base,
		Posts: postPathList,
	}
}

func convertPostName(postName string) string {
	filtered := s.Split(s.ReplaceAll(s.ToLower(postName), " ", "-"), ".")
	return filtered[0]
}
