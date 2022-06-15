package config

import s "strings"

func convertMarkdownToPostName(fileName string) string {
	return s.ReplaceAll(s.ToLower(fileName), " ", "-")
}
