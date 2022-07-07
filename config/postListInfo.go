package config

import (
	"io/ioutil"
	"log"
	s "strings"
)

type Post struct {
	DirPath       string
	DirName       string
	PostName      string
	AssetNameList []string
}

func (p Post) GetAssetPathList() []string {
	pathList := make([]string, len(p.AssetNameList))
	for index, asset := range p.AssetNameList {
		pathList[index] = p.DirPath + "/" + asset
	}
	return pathList
}

func (p Post) GetPostAbsolutePath() string {
	return p.DirPath + "/" + p.PostName
}

type PostListInfo struct {
	Base     string
	PostList []Post
}

func (yf YamlFile) ExtractPostListInfo() PostListInfo {
	postList := PostListInfo{}
	postList.Base = yf.Source.BaseDir
	draftPostsDir := yf.Source.BaseDir + "/" + yf.Source.DraftPostsDir
	posts, err := ioutil.ReadDir(draftPostsDir)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	postDirPathList := make([]Post, len(posts))
	for index, post := range posts {
		if post.IsDir() {
			currentPost := Post{}
			currentPost.DirPath = draftPostsDir + "/" + post.Name()
			currentPost.DirName = post.Name()
			postContents, err := ioutil.ReadDir(currentPost.DirPath)
			if err != nil {
				log.Fatalf("error: %v", err)
			}
			contentList := make([]string, len(postContents)-1)
			contentIndex := 0
			for _, content := range postContents {
				if s.HasSuffix(content.Name(), "md") {
					currentPost.PostName = content.Name()
				} else {
					contentList[contentIndex] = content.Name()
					contentIndex++
				}
			}
			currentPost.AssetNameList = contentList
			postDirPathList[index] = currentPost
		}
	}
	postList.PostList = postDirPathList
	return postList
}
