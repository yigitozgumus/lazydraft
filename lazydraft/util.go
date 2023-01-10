package lazydraft

import (
	"fmt"
	"io/ioutil"
	"os"
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

func CreateFileInUserHomeDir(filePath string, fileName string) {
	homeDir, err := os.UserHomeDir()
	filePath = homeDir + "/" + filePath
	_, err = os.ReadFile(filePath)
	if err != nil {
		fmt.Printf("\nCreating '%s'...", fileName)
		ioutil.WriteFile(filePath, []byte{}, 0666)
		fmt.Printf("\n'%s' is created at '%s'\n", fileName, filePath)
	} else {
		fmt.Printf("\n'%s' file is present.", fileName)
	}
}
