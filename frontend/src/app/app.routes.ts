import { Routes } from '@angular/router';
import { LoginComponent } from './components/login/login';

import { authGuard } from './auth/auth-guard';
import {LayoutComponent} from './components/layout/layout.component';

export const routes: Routes = [
  { path: '', redirectTo: '/dashboard', pathMatch: 'full' },
  { path: 'login', component: LoginComponent },
  {
    path: 'dashboard',
    component: LayoutComponent,
    canActivate: [authGuard]
  },
  { path: '**', redirectTo: '/dashboard' }
];
