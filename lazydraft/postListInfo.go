package lazydraft

import (
	"io/ioutil"
	"log"
	s "strings"
)

type PostListInfo struct {
	Base     string
	PostList []Post
}

func (pli PostListInfo) GetPostNameList() []string {
	postNames := make([]string, len(pli.PostList))
	for index, post := range pli.PostList {
		postNames[index] = post.PostName
	}
	return postNames
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
