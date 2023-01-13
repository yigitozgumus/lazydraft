package lazydraft

import (
	"fmt"
	"io/ioutil"
	"lazy-publish/util"
	"os"
	s "strings"

	"github.com/otiai10/copy"
)

type TargetInfo struct {
	Base        string
	ContentDir  string
	AssetDir    string
	AssetPrefix string
}

type Project struct {
	Name         string
	Posts        PostListInfo
	PublishedDir string
	Target       TargetInfo
}

func (p Project) CopyPostToTarget(postIndex int) error {
	postToCopy := p.Posts.PostList[postIndex]
	postAssetDir := p.Target.AssetDir
	for index, asset := range postToCopy.AssetNameList {
		fileToCopy, err := ioutil.ReadFile(postToCopy.GetAssetPathList()[index])
		util.HandleError(err)
		ioutil.WriteFile(postAssetDir+"/"+asset, fileToCopy, 0666)
	}
	postContent, err := ioutil.ReadFile(postToCopy.GetPostAbsolutePath())
	util.HandleError(err)
	targetAssetPrefix := "/" + p.Target.AssetPrefix
	updatedContent := s.ReplaceAll(string(postContent), "assets", targetAssetPrefix)
	postFileName := p.Target.ContentDir + "/" + util.ConvertMarkdownToPostName(postToCopy.PostName)
	ioutil.WriteFile(postFileName, []byte(updatedContent), 0666)
	return nil
}

func (p *Project) RemovePostFromTarget(draft Post) error {
	postAssetDir := p.Target.AssetDir
	for _, asset := range draft.AssetNameList {
		err := os.Remove(postAssetDir + "/" + asset)
		util.HandleError(err)
	}

	postFileName := p.Target.ContentDir + "/" + util.ConvertMarkdownToPostName(draft.PostName)
	err := os.Remove(postFileName)
	util.HandleError(err)
	return nil
}

func (p *Project) UpdatePostToLatest(draft Post, index int) error {
	err := p.RemovePostFromTarget(draft)
	util.HandleError(err)
	err = p.CopyPostToTarget(index)
	util.HandleError(err)
	fmt.Println("\n • Post is updated")
	return nil
}

func (p *Project) RemovePostFromDrafts(draft Post) error {
	err := os.Remove(draft.GetPostAbsolutePath())
	util.HandleError(err)
	fmt.Println(" • Post is removed from drafts directory")
	return nil
}

func (p *Project) CopyDraftToPublished(draft Post) error {
	publishedDirPath := p.PublishedDir + "/" + draft.PostName
	draftDirPath := draft.GetPostAbsolutePath()
	err := copy.Copy(draftDirPath, publishedDirPath)
	util.HandleError(err)
	fmt.Println(" • Post is copied to published directory")
	return nil
}

func (p *Project) GetStagedPosts() ([]Post, error) {
	targetFiles, err := p.GetTargetContentDirFiles()
	util.HandleError(err)
	draftList := p.Posts.PostList
	stagedDrafts := make([]Post, 0)
	for _, draft := range draftList {
		for _, target := range targetFiles {
			if target == util.ConvertMarkdownToPostName(draft.PostName) {
				stagedDrafts = append(stagedDrafts, draft)
			}
		}
	}
	return stagedDrafts, nil
}

func (p Project) GetTargetContentDirFiles() ([]string, error) {
	targetContent := p.Target.ContentDir
	files, err := ioutil.ReadDir(targetContent)
	util.HandleError(err)
	fileNames := make([]string, len(files))
	for index, file := range files {
		fileNames[index] = file.Name()
	}
	return fileNames, nil
}
