package lazydraft

type YamlFile struct {
	Source struct {
		BaseDir      string `yaml:"base_dir"`
		DraftDir     string `yaml:"draft_posts_dir"`
		PublishedDir string `yaml:"published_posts_dir"`
		AssetsDir    string `yaml:"assets_dir"`
	}
	Target struct {
		BaseDir     string `yaml:"base_dir"`
		ContentDir  string `yaml:"content_dir"`
		AssetDir    string `yaml:"assets_dir"`
		AssetPrefix string `yaml:"asset_prefix"`
	}
}

func (yf YamlFile) ExtractTargetListInfo() TargetInfo {
	target := TargetInfo{}
	target.Base = yf.Target.BaseDir
	target.AssetDir = yf.Target.BaseDir + "/" + yf.Target.AssetDir
	target.ContentDir = yf.Target.BaseDir + "/" + yf.Target.ContentDir
	target.AssetPrefix = yf.Target.AssetPrefix
	return target
}

func (yf YamlFile) ExtractTargetPublishedDir() string {
	return yf.Source.BaseDir + "/" + yf.Source.PublishedDir
}
