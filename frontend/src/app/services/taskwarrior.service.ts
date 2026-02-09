import {Injectable} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {Router} from '@angular/router';
import {Observable} from 'rxjs';
import {Task} from '../models/task.model';

@Injectable({
  providedIn: 'root'
})
export class TaskwarriorService {
  private apiUrl = 'http://localhost:8090/api';

  constructor(private http: HttpClient, private router: Router) {}

  getTasks(): Observable<Task[]> {
    return this.http.get<Task[]>(`${this.apiUrl}/tasks`);
  }

  getProjects(): Observable<Task[]> {
    return this.http.get<Task[]>(`${this.apiUrl}/projects`);
  }

  getTags(): Observable<Task[]> {
    return this.http.get<Task[]>(`${this.apiUrl}/tags`);
  }

  getTaskById(uuid: string): Observable<Task> {
    return this.http.get<Task>(`${this.apiUrl}/tasks/${uuid}`);
  }

  updateTaskStatus(uuid: string, status: string): Observable<Task> {
    return this.http.patch<Task>(`${this.apiUrl}/tasks/${uuid}`, { status });
  }

  syncTasks() {
    return this.http.post(`${this.apiUrl}/sync`,{});
  }
}
