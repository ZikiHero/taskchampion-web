import { HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { AuthService } from '../services/auth.service';

// auth.interceptor.ts
export const authInterceptor: HttpInterceptorFn = (req, next) => {
  console.log('🔄 Interceptor called for:', req.url);

  const authService = inject(AuthService);
  const token = authService.getToken();

  console.log('🔑 Token in interceptor:', token ? 'EXISTS' : 'MISSING');

  // Don't add token to auth endpoints
  if (req.url.includes('/api/login') ||
    req.url.includes('/api/register')) {
    console.log('⏭️ Skipping auth endpoints');
    return next(req);
  }

  // Add token to other requests
  if (token) {
    console.log('✅ Adding token to request');
    const cloned = req.clone({
      headers: req.headers.set('Authorization', `Bearer ${token}`)
    });
    console.log('📤 Request headers:', cloned.headers.get('Authorization'));
    return next(cloned);
  }

  console.warn('⚠️ No token found, sending request without auth');
  return next(req);
};

