import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router } from '@angular/router';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatDividerModule } from '@angular/material/divider';
import { AuthService } from '../../services/auth.service';
import {TaskwarriorService} from '../../services/taskwarrior.service';
import {Task} from '../../models/task.model';
import {MatTooltip} from '@angular/material/tooltip';
import {MatChip} from '@angular/material/chips';
import {
  MatCell, MatCellDef, MatColumnDef,
  MatHeaderCell, MatHeaderCellDef,
  MatHeaderRow,
  MatHeaderRowDef,
  MatRow,
  MatRowDef,
  MatTable
} from '@angular/material/table';
import {MatProgressSpinner} from '@angular/material/progress-spinner';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [
    CommonModule,
    MatToolbarModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatMenuModule,
    MatDividerModule,
    MatTooltip,
    MatChip,
    MatCell,
    MatHeaderCell,
    MatTable,
    MatProgressSpinner,
    MatRow,
    MatHeaderRow,
    MatHeaderRowDef,
    MatRowDef,
    MatHeaderCellDef,
    MatCellDef,
    MatColumnDef
  ],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.scss'
})
export class DashboardComponent implements OnInit {
  userEmail: string = '';
  userRole: string = '';

  tasks: Task[] = [];
  isLoadingTasks = false;
  displayedColumns: string[] = ['status', 'description', 'project', 'due', 'actions'];

  constructor(
    private authService: AuthService,
    private taskService: TaskwarriorService,
    private router: Router
  ) {}

  ngOnInit(): void {
    const user = this.authService.getCurrentUser();
    if (user) {
      this.userEmail = user.email;
      this.userRole = user.role;
    }

    this.loadTasks();
  }

  logout(): void {
    this.authService.logout();
  }

  private loadTasks(): void {
    this.isLoadingTasks = true;
    this.taskService.getTasks().subscribe({
      next: (tasks) => {
        this.tasks = tasks;
        this.isLoadingTasks = false;
      },
      error: (error) => {
        console.error('Error loading tasks:', error);
        this.isLoadingTasks = false;
      }
    });
  }

  getStatusColor(status: string): string {
    switch (status) {
      case 'pending': return 'warn';
      case 'in_progress': return 'accent';
      case 'completed': return 'primary';
      default: return '';
    }
  }

  getStatusIcon(status: string): string {
    switch (status) {
      case 'pending': return 'schedule';
      case 'in_progress': return 'hourglass_empty';
      case 'completed': return 'check_circle';
      default: return 'help';
    }
  }

  formatDate(dateString: string): string {
    if (!dateString) return '-';
    // Parse Taskwarrior format: 20250918T145019Z
    const year = dateString.substring(0, 4);
    const month = dateString.substring(4, 6);
    const day = dateString.substring(6, 8);
    return `${day}.${month}.${year}`;
  }

  extractTaskLink(description: string): string | null {
    const urlMatch = description.match(/(https?:\/\/[^\s]+)/);
    return urlMatch ? urlMatch[1] : null;
  }

  extractTaskTitle(description: string): string {
    // Entferne (bw)#... prefix und URL
    return description
      .replace(/^\(bw\)#[^\s]+\s*-\s*/, '')
      .replace(/\s*\.\.?\s*https?:\/\/[^\s]+$/, '');
  }

  openTaskLink(description: string): void {
    const link = this.extractTaskLink(description);
    if (link) {
      window.open(link, '_blank');
    }
  }

  syncTasks(): void {
    this.taskService.syncTasks().subscribe({
      next: () => {
        this.loadTasks();
      },
      error: (error) => {
        console.error('Error syncing tasks:', error);
      }
    });
  }

  updateStatus(task: Task, newStatus: string): void {
    /*this.taskService.updateTaskStatus(task.uuid, newStatus).subscribe({
      next: () => {
        task.status = newStatus as any;
      },
      error: (error) => {
        console.error('Error updating task:', error);
      }
    });*/
  }
}
