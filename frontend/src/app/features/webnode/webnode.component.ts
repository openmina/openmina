import { ChangeDetectionStrategy, Component } from '@angular/core';
import {
  WebNodeDemoDashboardComponent,
} from '@app/features/webnode/web-node-demo-dashboard/web-node-demo-dashboard.component';

@Component({
  selector: 'mina-webnode',
  standalone: true,
  imports: [
    WebNodeDemoDashboardComponent,
  ],
  templateUrl: './webnode.component.html',
  styleUrl: './webnode.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WebnodeComponent {

}
