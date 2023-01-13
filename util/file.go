package util

import (
	"fmt"
	"io/ioutil"
	"os"
)

func CreateFileInUserHomeDir(filePath string, fileName string) {
	homeDir, err := os.UserHomeDir()
	HandleError(err)
	filePath = homeDir + "/" + filePath
	_, err = os.ReadFile(filePath)
	if err != nil {
		fmt.Printf("\nCreating '%s'...\n", fileName)
		ioutil.WriteFile(filePath, []byte{}, 0666)
		fmt.Printf(" • '%s' is created at '%s'\n", fileName, filePath)
	} else {
		fmt.Printf(" • '%s' file is present.\n", fileName)
	}
}
