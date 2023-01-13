package util

func CheckErrorAndReturn(err error) error {
	if err != nil {
		return err
	}
	return nil
}
