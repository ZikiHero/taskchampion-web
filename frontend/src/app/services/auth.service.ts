import {Injectable} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {Observable, tap, catchError, throwError, BehaviorSubject, finalize, of} from 'rxjs';
import {LoginRequest} from '../models/login-request.model';
import {LoginResponse} from '../models/login-response.model';
import {User} from '../models/user.model';
import {Router} from '@angular/router';
import {environment} from '../../environments/environment'

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private apiUrl = environment.apiUrl;
  private currentUserSubject = new BehaviorSubject<User | null>(this.getCurrentUser());
  public currentUser$ = this.currentUserSubject.asObservable();

  constructor(private http: HttpClient, private router: Router) {}

  login(credentials: LoginRequest): Observable<LoginResponse> {
    console.log('🔐 Login attempt:', credentials.email);

    return this.http.post<LoginResponse>(`${this.apiUrl}/login`, credentials)
      .pipe(
        tap(response => {
          console.log('✅ Login successful:', response.email);
          this.setSession(response);
          this.currentUserSubject.next({
            id: response.id,
            email: response.email,
            role: response.role
          });
        }),
        catchError(error => {
          console.error('❌ Login failed:', error);
          return throwError(() => error);
        })
      );
  }

  logout(): void {
    console.log('🚪 Logout initiated');
    this.http.post<void>(`${this.apiUrl}/logout`, {})
      .pipe(
        tap(() => console.log('✅ Backend logout successful')),
        catchError(err => {
          console.warn('⚠️ Backend logout failed:', err);
          return of(null);
        }),
        finalize(() => {
          console.log('🧹 Cleaning up session');
          this.clearSession();
          this.router.navigate(['/login']);
        })
      ).subscribe();
  }


  isLoggedIn(): boolean {
    return !!localStorage.getItem('token');
  }

  getUserId(): string | null {
    return localStorage.getItem('id');
  }

  getToken(): string | null {
    return localStorage.getItem('token');
  }

  isAuthenticated(): boolean {
    return !!localStorage.getItem('token');
  }

  getCurrentUser(): User | null {

    if (!this.isLoggedIn()) {
      return null;
    }

    const id = localStorage.getItem('id');
    const email = localStorage.getItem('email');
    const role = localStorage.getItem('role');

    if (!email || !role || !id) {
      return null;
    }

    return { id, email, role };
  }

  private clearSession(): void {
    console.log('🗑️ Clearing session');

    localStorage.removeItem('token');
    //localStorage.removeItem('refresh_token');
    //localStorage.removeItem('expires_at');
    localStorage.removeItem('email');
    localStorage.removeItem('role');

    this.currentUserSubject.next(null);

    console.log('✅ Session cleared');
  }

  private setSession(authResult: LoginResponse): void {
    localStorage.setItem('id', authResult.id);
    localStorage.setItem('token', authResult.token);
    localStorage.setItem('email', authResult.email);
    localStorage.setItem('role', authResult.role);
  }
}
