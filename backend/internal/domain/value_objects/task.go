package value_objects

import (
	"errors"
	"fmt"
	"time"

	"hufschlaeger.net/tcweb-backend/internal/domain"
)

type TaskUUID struct {
	value string
}

func NewTaskUUID(uuid string) (TaskUUID, error) {
	if uuid == "" {
		return TaskUUID{}, domain.ErrInvalidUUID
	}
	return TaskUUID{value: uuid}, nil
}

func (u TaskUUID) String() string {
	return u.value
}

func (u TaskUUID) Equals(other TaskUUID) bool {
	return u.value == other.value
}

type Description struct {
	value string
}

func NewDescription(text string) (Description, error) {
	if text == "" {
		return Description{}, errors.New("beschreibung darf nicht leer sein")
	}
	return Description{value: text}, nil
}

func (d Description) String() string {
	return d.value
}

type TaskStatus string

const (
	StatusPending   TaskStatus = "pending"
	StatusCompleted TaskStatus = "completed"
	StatusDeleted   TaskStatus = "deleted"
	StatusWaiting   TaskStatus = "waiting"
)

func NewTaskStatus(status string) (TaskStatus, error) {
	s := TaskStatus(status)
	switch s {
	case StatusPending, StatusCompleted, StatusDeleted, StatusWaiting:
		return s, nil
	default:
		return "", fmt.Errorf("ungültiger Status: %s", status)
	}
}

type Priority string

const (
	PriorityLow    Priority = "L"
	PriorityMedium Priority = "M"
	PriorityHigh   Priority = "H"
)

func NewPriority(p string) (Priority, error) {
	if p == "" {
		return "", errors.New("priorität darf nicht leer sein")
	}
	priority := Priority(p)
	switch priority {
	case PriorityLow, PriorityMedium, PriorityHigh:
		return priority, nil
	default:
		return "", fmt.Errorf("ungültige Priorität: %s", p)
	}
}

type Tag struct {
	value string
}

func NewTag(tag string) (Tag, error) {
	if tag == "" {
		return Tag{}, errors.New("tag darf nicht leer sein")
	}
	return Tag{value: tag}, nil
}

func (t Tag) String() string {
	return t.value
}

func (t Tag) Equals(other Tag) bool {
	return t.value == other.value
}

type Project struct {
	value string
}

func NewProject(project string) (Project, error) {
	if project == "" {
		return Project{}, errors.New("projekt darf nicht leer sein")
	}
	return Project{value: project}, nil
}

func (p Project) String() string {
	return p.value
}

type Task struct {
	uuid        TaskUUID
	description Description
	status      TaskStatus
	entry       time.Time
	modified    time.Time
	tags        []Tag
	priority    *Priority
	due         *time.Time
	project     *Project
}

func NewTask(
	uuid TaskUUID,
	description Description,
	status TaskStatus,
	entry time.Time,
) (*Task, error) {
	return &Task{
		uuid:        uuid,
		description: description,
		status:      status,
		entry:       entry,
		modified:    time.Now(),
		tags:        []Tag{},
	}, nil
}

func (t *Task) Complete() error {
	if t.status == StatusCompleted {
		return domain.ErrTaskAlreadyCompleted
	}
	t.status = StatusCompleted
	t.modified = time.Now()
	return nil
}

func (t *Task) SetPriority(p Priority) {
	t.priority = &p
	t.modified = time.Now()
}

func (t *Task) SetModified(modified time.Time) {
	t.modified = modified
}

func (t *Task) SetDue(due time.Time) {
	t.due = &due
	t.modified = time.Now()
}

func (t *Task) AddTag(tag Tag) error {
	// Prüfen ob Tag bereits existiert
	for _, existingTag := range t.tags {
		if existingTag.Equals(tag) {
			return domain.ErrTagAlreadyExists
		}
	}
	t.tags = append(t.tags, tag)
	t.modified = time.Now()
	return nil
}

func (t *Task) SetProject(project Project) {
	t.project = &project
	t.modified = time.Now()
}

func (t *Task) UUID() TaskUUID           { return t.uuid }
func (t *Task) Description() Description { return t.description }
func (t *Task) Status() TaskStatus       { return t.status }
func (t *Task) Entry() time.Time         { return t.entry }
func (t *Task) Modified() time.Time      { return t.modified }
func (t *Task) Tags() []Tag              { return t.tags }
func (t *Task) Priority() *Priority      { return t.priority }
func (t *Task) Due() *time.Time          { return t.due }
func (t *Task) Project() *Project        { return t.project }
