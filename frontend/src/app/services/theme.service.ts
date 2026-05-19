import { Injectable, signal, effect } from '@angular/core';

export type ThemeMode = 'light' | 'dark' | 'system';

@Injectable({
  providedIn: 'root'
})
export class ThemeService {
  private _theme = signal<ThemeMode>((localStorage.getItem('theme') as ThemeMode) || 'system');
  theme = this._theme.asReadonly();

  constructor() {
    effect(() => {
      const mode = this._theme();
      localStorage.setItem('theme', mode);
      this.applyTheme(mode);
    });

    // Listen for system theme changes if in 'system' mode
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (this._theme() === 'system') {
        this.applyTheme('system');
      }
    });
  }

  setTheme(mode: ThemeMode) {
    this._theme.set(mode);
  }

  private applyTheme(mode: ThemeMode) {
    const isDark = mode === 'dark' ||
      (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

    if (isDark) {
      document.body.classList.add('dark-theme');
      document.body.classList.remove('light-theme');
    } else {
      document.body.classList.add('light-theme');
      document.body.classList.remove('dark-theme');
    }
  }
}
