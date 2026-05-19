import { Component, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatListModule } from '@angular/material/list';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatDialogModule, MatDialog } from '@angular/material/dialog';
import { TaskListComponent } from '../task-list/task-list.component';
import { SettingsDialogComponent } from '../settings-dialog/settings-dialog.component';
import { TaskService } from '../../services/task.service';
import { ThemeService } from '../../services/theme.service';
import { ViewType } from '../../models/task.model';

@Component({
  selector: 'app-layout',
  standalone: true,
  imports: [
    CommonModule,
    MatToolbarModule,
    MatSidenavModule,
    MatListModule,
    MatButtonModule,
    MatIconModule,
    MatTooltipModule,
    MatDialogModule,
    TaskListComponent
  ],
  templateUrl: './layout.component.html',
  styleUrls: ['./layout.component.scss']
})
export class LayoutComponent {
  taskService = inject(TaskService);
  themeService = inject(ThemeService);
  dialog = inject(MatDialog);

  isLoggedIn = signal(false);
  userName = signal('User Champion');

  openSettings() {
    this.dialog.open(SettingsDialogComponent, {
      width: '500px',
      autoFocus: false
    });
  }

  login() {
    this.isLoggedIn.set(true);
  }

  logout() {
    this.isLoggedIn.set(false);
  }

  getInboxCount() {
    return this.taskService.tasks().filter(t => !t.projectId && !t.isCompleted).length;
  }

  getTodayCount() {
    const today = new Date().toISOString().split('T')[0];
    return this.taskService.tasks().filter(t => t.dueDate === today && !t.isCompleted).length;
  }

  getProjectCount(projectId: string) {
    return this.taskService.tasks().filter(t => t.projectId === projectId && !t.isCompleted).length;
  }

  getTagCount(tag: string) {
    return this.taskService.tasks().filter(t => t.tags?.includes(tag) && !t.isCompleted).length;
  }
}
