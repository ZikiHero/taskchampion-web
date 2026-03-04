export const environment = {
  production: true,
  apiUrl: (window as any).ENV?.apiUrl ?? 'http://localhost:8080/api'
};
