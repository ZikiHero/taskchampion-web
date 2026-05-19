import { Injectable, signal, computed } from '@angular/core';
import { Task, Project, ViewType } from '../models/task.model';

@Injectable({
  providedIn: 'root'
})
export class TaskService {
  private _tasks = signal<Task[]>([
    {
      id: '1',
      content: 'Buy groceries',
      isCompleted: false,
      priority: 'H',
      createdAt: new Date(),
      dueDate: new Date().toISOString().split('T')[0],
      tags: ['Shopping', 'Home']
    },
    {
      id: '2',
      content: 'Finish new Taskchamp webUI',
      description: 'Complete the frontend part with Angular 20',
      isCompleted: false,
      priority: 'L',
      createdAt: new Date(),
      dueDate: new Date().toISOString().split('T')[0],
      projectId: 'p2',
      tags: ['Work', 'Angular']
    },
    {
      id: '3',
      content: 'Morning exercise',
      isCompleted: true,
      priority: 'M',
      createdAt: new Date()
    }
  ]);

  private _projects = signal<Project[]>([
    { id: 'p1', name: 'Personal', color: '#ffcc00' },
    { id: 'p2', name: 'Work', color: '#058527' }
  ]);

  private _tags = signal<string[]>(['Shopping', 'Home', 'Work', 'Angular']);

  private _currentView = signal<ViewType>('inbox');

  tasks = this._tasks.asReadonly();
  projects = this._projects.asReadonly();
  tags = this._tags.asReadonly();
  currentView = this._currentView.asReadonly();

  filteredTasks = computed(() => {
    const allTasks = this._tasks();
    const view = this._currentView();

    if (view === 'inbox') {
      return allTasks.filter(t => !t.projectId);
    } else if (view === 'today') {
      const today = new Date().toISOString().split('T')[0];
      return allTasks.filter(t => t.dueDate === today);
    } else if (view === 'upcoming') {
      const today = new Date().toISOString().split('T')[0];
      return allTasks.filter(t => t.dueDate && t.dueDate > today);
    } else {
      // project filtering
      return allTasks.filter(t => t.projectId === view);
    }
  });

  setView(view: ViewType) {
    this._currentView.set(view);
  }


  addTask(task: Omit<Task, 'id' | 'isCompleted' | 'createdAt'>) {
    const newTask: Task = {
      ...task,
      id: Math.random().toString(36).substring(2),
      isCompleted: false,

      createdAt: new Date()
    };
    this._tasks.update(tasks => [...tasks, newTask]);
  }

  updateTask(id: string, updates: Partial<Task>) {
    this._tasks.update(tasks => tasks.map(t =>
      t.id === id ? { ...t, ...updates } : t
    ));
  }

  toggleTask(id: string) {
    this._tasks.update(tasks => tasks.map(t =>
      t.id === id ? { ...t, isCompleted: !t.isCompleted } : t
    ));
  }

  deleteTask(id: string) {
    this._tasks.update(tasks => tasks.filter(t => t.id !== id));
  }

  addProject(name: string, color: string = '#808080') {
    const newProject: Project = {
      id: 'p' + Math.random().toString(36).substring(2, 9),
      name,
      color
    };
    this._projects.update(projects => [...projects, newProject]);
    return newProject;
  }
}
