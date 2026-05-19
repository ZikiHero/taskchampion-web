# Taskchampion Web Frontend

A modern, responsive web interface for managing Taskwarrior tasks, built with Angular and Angular Material.

## Features

- **Task Dashboard**: View, filter, and manage your tasks.
- **Task Management**: Create, edit, and delete tasks with ease.
- **Project & Tag Views**: Organize your tasks by projects and tags.
- **Responsive Design**: Optimized for both desktop and mobile use.
- **Real-time Updates**: Synchronizes with the backend to keep your tasks up to date.

## Technology Stack

- **Framework**: [Angular](https://angular.dev/) 21
- **UI Components**: [Angular Material](https://material.angular.io/)
- **Styling**: SCSS
- **State Management**: RxJS

## Prerequisites

- [Node.js](https://nodejs.org/) (v24 recommended)
- [npm](https://www.npmjs.com/)
- [Angular CLI](https://angular.dev/tools/cli)

## Getting Started

### Local Development

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Start the development server**:
   ```bash
   npm start
   ```
   Navigate to `http://localhost:4200/`. The app will automatically reload if you change any of the source files.

### Available Scripts

- `npm start`: Runs the app in development mode.
- `npm run build`: Builds the app for production to the `dist/` directory.
- `npm test`: Executes unit tests via [Karma](https://karma-runner.github.io).
- `npm run watch`: Builds the app in development mode and watches for changes.

## Docker

To build and run the frontend using Docker:

1. **Build the image**:
   ```bash
   docker build -t taskchampion-frontend .
   ```

2. **Run the container**:
   ```bash
   docker run -p 4200:80 taskchampion-frontend
   ```

### DevContainer

A DevContainer configuration is provided for a seamless development experience.

#### Quick Start with VS Code
1. Install the **Dev Containers** extension.
2. Open the `frontend` folder or the project root in VS Code.
3. Use the command `Dev Containers: Reopen in Container`.

#### Podman Support
To use Podman instead of Docker:
- Set `"dev.containers.dockerPath": "podman"` in your VS Code `settings.json`.
- On Linux with SELinux, you may need to add `"runArgs": ["--security-opt", "label=disable"]` to `.devcontainer/devcontainer.json`.
- If you use `podman-compose`, also set `"dev.containers.dockerComposePath": "podman-compose"`.

## Project Structure

- `src/app/models`: TypeScript models and interfaces.
- `src/app/services`: Data services for API communication.
- `src/app/components`: Reusable UI components.
- `src/app/pages`: Main application views/pages.
- `src/assets`: Static assets like images and global styles.

## Configuration

The frontend communicates with the backend API. In development, you can configure the API proxy in `proxy.conf.json` (if present) or via environment files in `src/environments/`.
