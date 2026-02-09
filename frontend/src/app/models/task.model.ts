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
