package lazydraft

import (
	"fmt"
	"github.com/otiai10/copy"
	"io/ioutil"
	"os"
	s "strings"
)

const ObsidianImgPrefix = "![["
const MarkdownImgPrefix = "![]("
const ImgClosure = "]]"
const ImgPrefix = "/img/"

type TargetInfo struct {
	Base       string
	ContentDir string
	AssetDir   string
}

type Project struct {
	Name         string
	Posts        PostListInfo
	PublishedDir string
	Target       TargetInfo
}

func (p Project) CopyPostToTarget(postIndex int) error {
	postToCopy := p.Posts.PostList[postIndex]
	postAssetDir := p.Target.AssetDir + "/" + postToCopy.BaseDir
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
	fullPrefix := ImgPrefix + postToCopy.BaseDir + "/"
	updatedContent := s.ReplaceAll(string(postContent), ObsidianImgPrefix, MarkdownImgPrefix+fullPrefix)
	updatedContent = s.ReplaceAll(updatedContent, ImgClosure, ")")
	postFileName := p.Target.ContentDir + "/" + ConvertMarkdownToPostName(postToCopy.PostName)
	ioutil.WriteFile(postFileName, []byte(updatedContent), 0666)
	return nil
}

func (p *Project) RemovePostFromTarget(draft Post) error {
	postAssetDir := p.Target.AssetDir + "/" + draft.BaseDir
	err := os.RemoveAll(postAssetDir)
	if err != nil {
		return err
	}
	postFileName := p.Target.ContentDir + "/" + ConvertMarkdownToPostName(draft.PostName)
	err = os.Remove(postFileName)
	if err != nil {
		return err
	}
	return nil
}

func (p *Project) UpdatePostToLatest(draft Post, index int) error {
	err := p.RemovePostFromTarget(draft)
	if err != nil {
		return err
	}
	err = p.CopyPostToTarget(index)
	if err != nil {
		return err
	}
	fmt.Println("\n • Post is updated")
	return nil
}

func (p *Project) RemovePostFromDrafts(draft Post) error {
	draftDirPath := draft.BaseDir
	err := os.RemoveAll(draftDirPath)
	if err != nil {
		return err
	}
	fmt.Println(" • Post is removed from drafts directory")
	return nil
}

func (p *Project) CopyDraftToPublished(draft Post) error {
	publishedDirPath := p.PublishedDir + "/" + draft.BaseDir
	draftDirPath := draft.BaseDir
	err := copy.Copy(draftDirPath, publishedDirPath)
	if err != nil {
		return err
	}
	fmt.Println(" • Post is copied to published directory")
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
	targetContent := p.Target.ContentDir
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
