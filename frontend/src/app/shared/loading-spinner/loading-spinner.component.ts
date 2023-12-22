import { ChangeDetectionStrategy, Component } from '@angular/core';

@Component({
  selector: 'mina-loading-spinner',
  standalone: true,
  templateUrl: './loading-spinner.component.html',
  styleUrls: ['./loading-spinner.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LoadingSpinnerComponent {
}
