import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, FormGroup, Validators, ReactiveFormsModule } from '@angular/forms';
import { Router } from '@angular/router';

// Angular Material Imports
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';

import { AuthService } from '../../services/auth.service';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
    MatCheckboxModule,
    MatProgressSpinnerModule,
    MatSnackBarModule
  ],
  templateUrl: 'login.html',
  styleUrl: './login.scss'
})
export class LoginComponent {
  loginForm: FormGroup;
  errorMessage: string = "";
  isLoading = false;
  hidePassword = true;

  constructor(
    private fb: FormBuilder,
    private authService: AuthService,
    private router: Router,
    private snackBar: MatSnackBar
  ) {
    this.loginForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', [Validators.required, Validators.minLength(8)]]
    });
  }

  onSubmit(): void {
    console.log('🔐 Submit clicked'); // ✅ Debug
    console.log('📋 Form valid:', this.loginForm.valid); // ✅ Debug
    console.log('📋 Form value:', this.loginForm.value); // ✅ Debug

    // ✅ Prüfe ob Form invalid
    if (this.loginForm.invalid) {
      console.log('❌ Form invalid, marking all as touched');
      this.loginForm.markAllAsTouched();
      return;
    }

    this.isLoading = true;
    this.errorMessage = '';

    console.log('🚀 Calling AuthService.login()'); // ✅ Debug

    // ✅ Extrahiere Werte explizit
    const credentials = {
      email: this.loginForm.get('email')?.value,
      password: this.loginForm.get('password')?.value
    };

    console.log('📦 Credentials:', credentials); // ✅ Debug

    this.authService.login(credentials).subscribe({
      next: (response) => {
        console.log('✅ Login success:', response); // ✅ Debug
        this.isLoading = false;

        this.snackBar.open('Login successful!', 'Close', {
          duration: 3000,
          horizontalPosition: 'end',
          verticalPosition: 'top',
          panelClass: ['success-snackbar']
        });

        console.log('🚀 Navigating to /dashboard'); // ✅ Debug

        // ✅ Navigation mit then() Chain
        this.router.navigate(['/dashboard']).then(
          (success) => {
            console.log('Navigation success:', success);
          },
          (error) => {
            console.error('Navigation error:', error);
          }
        );
      },
      error: (error) => {
        console.error('❌ Login error:', error); // ✅ Debug
        this.isLoading = false;

        // ✅ Detaillierte Fehlermeldung
        const errorMsg = error.error?.message
          || error.message
          || 'Login failed. Please check your credentials.';

        this.errorMessage = errorMsg;

        this.snackBar.open(errorMsg, 'Close', {
          duration: 5000,
          horizontalPosition: 'end',
          verticalPosition: 'top',
          panelClass: ['error-snackbar']
        });
      }
    });
  }

  getEmailErrorMessage(): string {
    const emailControl = this.loginForm.get('email');
    if (emailControl?.hasError('required')) {
      return 'Email is required';
    }
    return emailControl?.hasError('email') ? 'Not a valid email' : '';
  }

  getPasswordErrorMessage(): string {
    const passwordControl = this.loginForm.get('password');
    if (passwordControl?.hasError('required')) {
      return 'Password is required';
    }
    return passwordControl?.hasError('minlength')
      ? 'Password must be at least 8 characters'
      : '';
  }
}
