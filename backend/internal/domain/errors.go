package domain

import "errors"

var (
	ErrTaskAlreadyCompleted = errors.New("task ist bereits abgeschlossen")
	ErrTagAlreadyExists     = errors.New("tag existiert bereits")
	ErrInvalidUUID          = errors.New("ungültige UUID")
)
