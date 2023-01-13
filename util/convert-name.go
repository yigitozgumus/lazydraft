package util

import s "strings"

func ConvertMarkdownToPostName(fileName string) string {
	return s.ReplaceAll(s.ToLower(fileName), " ", "-")
}
