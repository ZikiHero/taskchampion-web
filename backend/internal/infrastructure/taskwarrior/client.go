package taskwarrior

import (
	"net/http"
	"time"
)

type TaskwarriorClient struct {
	baseURL    string
	httpClient *http.Client
	apiKey     string
}

func NewRestClient(baseURL, apiKey string) *TaskwarriorClient {
	return &TaskwarriorClient{
		baseURL: baseURL,
		apiKey:  apiKey,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
			Transport: &http.Transport{
				MaxIdleConns:        100,
				MaxIdleConnsPerHost: 100,
				IdleConnTimeout:     90 * time.Second,
			},
		},
	}
}
