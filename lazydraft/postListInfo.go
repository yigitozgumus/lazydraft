package lazydraft

import (
	"io/ioutil"
	"log"
	s "strings"
)

type Post struct {
	BaseDir       string
	AssetDir      string
	PostName      string
	AssetNameList []string
}

func (p Post) GetAssetPathList() []string {
	pathList := make([]string, len(p.AssetNameList))
	for index, asset := range p.AssetNameList {
		pathList[index] = p.AssetDir + "/" + asset
	}
	return pathList
}

func (p Post) GetPostAbsolutePath() string {
	return p.BaseDir + "/" + p.PostName
}

type PostListInfo struct {
	Base     string
	PostList []Post
}

func (yf YamlFile) ExtractPostListInfo() PostListInfo {
	postList := PostListInfo{}
	postList.Base = yf.Source.BaseDir
	draftPostsDir := yf.Source.BaseDir + "/" + yf.Source.DraftDir
	assetsDir := yf.Source.BaseDir + "/" + yf.Source.AssetsDir
	posts, err := ioutil.ReadDir(draftPostsDir)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	postDirPathList := make([]Post, len(posts))
	for index, post := range posts {
		currentPost := Post{}
		currentPost.BaseDir = draftPostsDir
		currentPost.PostName = post.Name()
		currentPost.AssetDir = assetsDir
		postAssets, err := ioutil.ReadDir(currentPost.AssetDir)
		if err != nil {
			log.Fatalf("error: %v", err)
		}
		// find asset prefix
		input, err := ioutil.ReadFile(currentPost.GetPostAbsolutePath())
		if err != nil {
			log.Fatalln(err)
		}
		lines := s.Split(string(input), "\n")
		var prefix string
		for _, line := range lines {
			if s.Contains(line, "asset-prefix:") {
				prefix = s.TrimSpace(s.Split(line, ":")[1])
			}
		}
		// find all asset files with prefix
		var assetList []string
		for _, content := range postAssets {
			if s.HasPrefix(content.Name(), prefix) {

				assetList = append(assetList, content.Name())
			}
		}
		currentPost.AssetNameList = assetList
		postDirPathList[index] = currentPost
	}
	postList.PostList = postDirPathList
	return postList
}
