package util

import (
	"fmt"
	"strconv"

	"github.com/manifoldco/promptui"
)

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

func GetSelectionFromPostList(inputList []string) (int, string, error) {
	prompt := promptui.Select{
		Label: "Choose",
		Items: inputList,
	}

	position, result, err := prompt.Run()
	if err != nil {
		fmt.Printf("Prompt failed %v\n", err)
		HandleError(err)
	}
	return position, result, nil
}
