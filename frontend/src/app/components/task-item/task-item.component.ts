import { Component, inject, Input, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { MatDividerModule } from '@angular/material/divider';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatNativeDateModule } from '@angular/material/core';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TaskService } from '../../services/task.service';
import { Task, Priority } from '../../models/task.model';

@Component({
  selector: 'app-task-item',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCheckboxModule,
    MatButtonModule,
    MatIconModule,
    MatInputModule,
    MatFormFieldModule,
    MatSelectModule,
    MatDividerModule,
    MatDatepickerModule,
    MatNativeDateModule,
    MatChipsModule,
    MatTooltipModule
  ],
  templateUrl: './task-item.component.html',
  styleUrls: ['./task-item.component.scss']
})
export class TaskItemComponent {
  taskService = inject(TaskService);
  @Input({ required: true }) task!: Task;

  isEditing = signal(false);
  editContent = '';
  editDescription = '';
  editPriority: Priority = 'L';
  editDueDate?: Date;
  editProjectId?: string;
  editTags: string[] = [];

  isAddingProject = signal(false);
  newProjectName = '';

  getPriorityColor() {
    switch (this.task.priority) {
      case 'H': return 'warn';
      case 'M': return 'accent';
      case 'L': return 'primary';
      default: return undefined;
    }
  }

  isOverdue() {
    if (!this.task.dueDate || this.task.isCompleted) return false;
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    return new Date(this.task.dueDate) < today;
  }

  getProjectName() {
    if (!this.task.projectId) return null;
    return this.taskService.projects().find(p => p.id === this.task.projectId)?.name;
  }

  startEdit() {
    this.editContent = this.task.content;
    this.editDescription = this.task.description || '';
    this.editPriority = this.task.priority;
    this.editDueDate = this.task.dueDate ? new Date(this.task.dueDate) : undefined;
    this.editProjectId = this.task.projectId;
    this.editTags = [...(this.task.tags || [])];
    this.isEditing.set(true);
  }

  saveEdit() {
    if (this.editContent.trim()) {
      this.taskService.updateTask(this.task.id, {
        content: this.editContent,
        description: this.editDescription,
        priority: this.editPriority,
        dueDate: this.editDueDate?.toISOString().split('T')[0],
        projectId: this.editProjectId,
        tags: this.editTags
      });
      this.isEditing.set(false);
    }
  }

  createNewProject() {
    if (this.newProjectName.trim()) {
      const project = this.taskService.addProject(this.newProjectName);
      this.editProjectId = project.id;
      this.newProjectName = '';
      this.isAddingProject.set(false);
    }
  }

  cancelEdit() {
    this.isEditing.set(false);
  }
}
