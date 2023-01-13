package util

import (
	"fmt"
	"strconv"
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
