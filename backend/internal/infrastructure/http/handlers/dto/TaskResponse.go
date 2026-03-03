package dto

type TaskResponse struct {
	UUID        string `json:"uuid"`
	Description string `json:"description"`
	Status      string `json:"status"`
	Entry       string `json:"entry"`
	Modified    string `json:"modified"`
	//Tags        []string `json:"tags"`
	Priority string `json:"priority"`
	Due      string `json:"due"`
	Project  string `json:"project"`
}
