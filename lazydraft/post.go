package lazydraft

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

func GetPostNameList(posts []Post) []string {
	nameList := make([]string, len(posts))
	for index, post := range posts {
		nameList[index] = post.PostName
	}
	return nameList
}

func (p Post) GetPostAbsolutePath() string {
	return p.BaseDir + "/" + p.PostName
}
