package config

import (
	"io/ioutil"
	"os"
	s "strings"
)

const ObsidianImgPrefix = "![["
const MarkdownImgPrefix = "![]("
const ImgClosure = "]]"
const ImgPrefix = "/img/"

type TargetInfo struct {
	TargetBase       string
	TargetContentDir string
	TargetAsset      string
}

type Project struct {
	Name   string
	Active bool
	Posts  PostListInfo
	Target TargetInfo
}

func (p Project) CopyPostToTarget(postIndex int) error {
	postToCopy := p.Posts.PostList[postIndex]
	postAssetDir := p.Target.TargetAsset + "/" + postToCopy.DirName
	err := os.Mkdir(postAssetDir, 0777)
	if err != nil {
		return err
	}
	for index, asset := range postToCopy.AssetNameList {
		fileToCopy, err := ioutil.ReadFile(postToCopy.GetAssetPathList()[index])
		if err != nil {
			return nil
		}
		ioutil.WriteFile(postAssetDir+"/"+asset, fileToCopy, 0666)
	}
	postContent, err := ioutil.ReadFile(postToCopy.GetPostAbsolutePath())
	if err != nil {
		return err
	}
	fullPrefix := ImgPrefix + postToCopy.DirName + "/"
	updatedContent := s.ReplaceAll(string(postContent), ObsidianImgPrefix, MarkdownImgPrefix+fullPrefix)
	updatedContent = s.ReplaceAll(updatedContent, ImgClosure, ")")
	postFileName := p.Target.TargetContentDir + "/" + ConvertMarkdownToPostName(postToCopy.PostName)
	ioutil.WriteFile(postFileName, []byte(updatedContent), 0666)
	return nil
}

func (p Project) RemovePostFromTarget(draft Post) error {
	postAssetDir := p.Target.TargetAsset + "/" + draft.DirName
	err := os.RemoveAll(postAssetDir)
	if err != nil {
		return err
	}
	postFileName := p.Target.TargetContentDir + "/" + ConvertMarkdownToPostName(draft.PostName)
	err = os.Remove(postFileName)
	if err != nil {
		return err
	}
	return nil
}

func (p *Project) GetStagedPosts() ([]Post, error) {
	targetFiles, err := p.GetTargetContentDirFiles()
	if err != nil {
		return nil, err
	}
	draftList := p.Posts.PostList
	stagedDrafts := make([]Post, 0)
	for _, draft := range draftList {
		for _, target := range targetFiles {
			if target == ConvertMarkdownToPostName(draft.PostName) {
				stagedDrafts = append(stagedDrafts, draft)
			}
		}
	}
	return stagedDrafts, nil
}

func (p Project) GetTargetContentDirFiles() ([]string, error) {
	targetContent := p.Target.TargetContentDir
	files, err := ioutil.ReadDir(targetContent)
	if err != nil {
		return nil, err
	}
	fileNames := make([]string, len(files))
	for index, file := range files {
		fileNames[index] = file.Name()
	}
	return fileNames, nil
}
