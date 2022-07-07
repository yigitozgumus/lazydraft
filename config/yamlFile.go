package config

type YamlFile struct {
	Source struct {
		BaseDir           string `yaml:"base_dir"`
		DraftPostsDir     string `yaml:"draft_posts_dir"`
		PublishedPostsDir string `yaml:"published_posts_dir"`
	}
	Target struct {
		BaseDir    string `yaml:"base_dir"`
		ContentDir string `yaml:"content_dir"`
		AssetDir   string `yaml:"asset_dir"`
	}
	Active bool `yaml:"active"`
}

func (yf YamlFile) ExtractTargetListInfo() TargetInfo {
	target := TargetInfo{}
	target.TargetBase = yf.Target.BaseDir
	target.TargetAsset = yf.Target.BaseDir + "/" + yf.Target.AssetDir
	target.TargetContentDir = yf.Target.BaseDir + "/" + yf.Target.ContentDir
	return target
}

func (yf YamlFile) ExtractTargetPublishedDir() string {
	return yf.Source.BaseDir + "/" + yf.Source.PublishedPostsDir
}
