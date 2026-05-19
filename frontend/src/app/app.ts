import { Component } from '@angular/core';
import {LayoutComponent} from './components/layout/layout.component';

/*@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  templateUrl: './app.html',
  styleUrl: './app.scss'
})*/

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [LayoutComponent],
  template: '<app-layout></app-layout>',
  styles: []
})
export class App {
  title = 'frontend';
}
