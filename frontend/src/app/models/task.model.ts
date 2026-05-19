/*
export interface Task {
  uuid: string;
  description: string;
  status: 'pending' | 'completed' | 'in_progress';
  entry: string;
  modified: string;
  priority: string;
  due: string;
  project: string;
}
*/
export type Priority = 'L' | 'M' | 'H' ;

export interface Task {
  id: string;
  content: string;
  description?: string;
  isCompleted: boolean;
  dueDate?: Date | string;
  priority: Priority;
  projectId?: string;
  tags?: string[];
  createdAt: Date;
}

export interface Project {
  id: string;
  name: string;
  color: string;
}

export type ViewType = 'inbox' | 'today' | 'upcoming' | string; // project id or view name
