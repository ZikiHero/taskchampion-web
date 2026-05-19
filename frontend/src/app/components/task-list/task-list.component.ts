import { Component, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { MatDividerModule } from '@angular/material/divider';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatNativeDateModule } from '@angular/material/core';
import { trigger, transition, style, animate, query, stagger } from '@angular/animations';
import { TaskService } from '../../services/task.service';
import { TaskItemComponent } from '../task-item/task-item.component';
import { Priority } from '../../models/task.model';

@Component({
  selector: 'app-task-list',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatButtonModule,
    MatIconModule,
    MatInputModule,
    MatFormFieldModule,
    MatSelectModule,
    MatDividerModule,
    MatDatepickerModule,
    MatNativeDateModule,
    TaskItemComponent
  ],
  animations: [
    trigger('listAnimation', [
      transition('* <=> *', [
        query(':enter', [
          style({ opacity: 0, transform: 'translateY(-10px)' }),
          stagger('50ms', animate('200ms ease-out', style({ opacity: 1, transform: 'translateY(0)' })))
        ], { optional: true }),
        query(':leave', [
          animate('200ms ease-in', style({ opacity: 0, transform: 'translateY(-10px)' }))
        ], { optional: true })
      ])
    ])
  ],
  templateUrl: './task-list.component.html',
  styleUrls: ['./task-list.component.scss']
})
export class TaskListComponent {
  taskService = inject(TaskService);
  showAddForm = signal(false);

  newTaskContent = '';
  newTaskDescription = '';
  newTaskPriority: Priority = 'L';
  newTaskDueDate?: Date;
  newTaskProjectId?: string;
  newTaskTags: string[] = [];

  isAddingProject = signal(false);
  newProjectName = '';

  listTitle = computed(() => {
    const view = this.taskService.currentView();
    if (view === 'inbox') return 'Inbox';
    if (view === 'today') return 'Today';
    if (view === 'upcoming') return 'Upcoming';

    const project = this.taskService.projects().find(p => p.id === view);
    return project ? project.name : 'Tasks';
  });

  addTask() {
    if (this.newTaskContent.trim()) {
      this.taskService.addTask({
        content: this.newTaskContent,
        description: this.newTaskDescription,
        priority: this.newTaskPriority,
        dueDate: this.newTaskDueDate?.toISOString().split('T')[0],
        projectId: this.newTaskProjectId,
        tags: this.newTaskTags
      });
      this.resetForm();
    }
  }

  createNewProject() {
    if (this.newProjectName.trim()) {
      const project = this.taskService.addProject(this.newProjectName);
      this.newTaskProjectId = project.id;
      this.newProjectName = '';
      this.isAddingProject.set(false);
    }
  }

  cancelAdd() {
    this.resetForm();
  }

  private resetForm() {
    this.newTaskContent = '';
    this.newTaskDescription = '';
    this.newTaskPriority = 'L';
    this.newTaskDueDate = undefined;
    this.newTaskProjectId = undefined;
    this.newTaskTags = [];
    this.showAddForm.set(false);
  }
}
