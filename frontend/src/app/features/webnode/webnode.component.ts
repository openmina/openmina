import { ChangeDetectionStrategy, Component } from '@angular/core';
import { WebNodeInitializationComponent } from '@app/features/webnode/web-node-initialization/web-node-initialization.component';

@Component({
  selector: 'mina-webnode',
  standalone: true,
  imports: [
    WebNodeInitializationComponent,
  ],
  templateUrl: './webnode.component.html',
  styleUrl: './webnode.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WebnodeComponent {

}
