package config

type YamlFile struct {
	Source struct {
		BaseDir  string `yaml:"base_dir"`
		PostsDir string `yaml:"posts_dir"`
	}
	Target struct {
		BaseDir    string `yaml:"base_dir"`
		ContentDir string `yaml:"content_dir"`
		AssetDir   string `yaml:"asset_dir"`
	}
}

type converter interface {
	ExtractPostListInfo() PostListInfo
	ExtractTargetListInfo() TargetInfo
}

func (yf YamlFile) ExtractTargetListInfo() TargetInfo {
	target := TargetInfo{}
	target.TargetBase = yf.Target.BaseDir
	target.TargetAsset = yf.Target.BaseDir + "/" + yf.Target.AssetDir
	target.TargetContentDir = yf.Target.BaseDir + "/" + yf.Target.ContentDir
	return target
}
