package lazypublish

import (
	"fmt"
	"strconv"
	s "strings"
)

func ConvertMarkdownToPostName(fileName string) string {
	return s.ReplaceAll(s.ToLower(fileName), " ", "-")
}

func GetInputFromUser(inputText string) (int, error) {
	var input string
	fmt.Printf("%s: ", inputText)
	fmt.Scanln(&input)
	inputInt, err := strconv.Atoi(input)
	if err != nil {
		return -1, err
	}
	return inputInt, nil
}
