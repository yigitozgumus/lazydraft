package config

import (
	"io/ioutil"
	"log"
	"os"
	s "strings"
)

const OBSIDIAN_IMG_PREFIX = "![["
const MARKDOWN_IMG_PREFIX = "![]("
const IMG_CLOSURE = "]]"
const IMG_PREFIX = "/img/"

type TargetInfo struct {
	TargetBase       string
	TargetContentDir string
	TargetAsset      string
}

type Project struct {
	Posts  PostListInfo
	Target TargetInfo
}

type projectOperations interface {
	InitProject(f InputFile)
	CopyPostToTarget(postIndex int)
}

func (p *Project) InitProject(f InputFile) {
	yamlFile := f.ReadYamlFile()
	(*p).Posts = yamlFile.ExtractPostListInfo()
	(*p).Target = yamlFile.ExtractTargetListInfo()
}

func (p Project) CopyPostToTarget(postIndex int) {
	postToCopy := p.Posts.PostList[postIndex]
	postAssetDir := p.Target.TargetAsset + "/" + postToCopy.DirName
	err := os.Mkdir(postAssetDir, 0777)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	for index, asset := range postToCopy.AssetNameList {
		fileToCopy, err := ioutil.ReadFile(postToCopy.GetAssetPathList()[index])
		if err != nil {
			log.Fatalf("error: %v", err)
		}
		ioutil.WriteFile(postAssetDir+"/"+asset, fileToCopy, 0666)
	}
	postContent, err := ioutil.ReadFile(postToCopy.GetPostAbsolutePath())
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	fullPrefix := IMG_PREFIX + postToCopy.DirName + "/"
	updatedContent := s.ReplaceAll(string(postContent), OBSIDIAN_IMG_PREFIX, MARKDOWN_IMG_PREFIX+fullPrefix)
	updatedContent = s.ReplaceAll(updatedContent, IMG_CLOSURE, ")")
	postFileName := p.Target.TargetContentDir + "/" + convertMarkdownToPostName(postToCopy.PostName)
	ioutil.WriteFile(postFileName, []byte(updatedContent), 0666)
}
