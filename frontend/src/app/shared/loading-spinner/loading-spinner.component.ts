import { ChangeDetectionStrategy, Component, Input } from '@angular/core';

@Component({
  selector: 'mina-loading-spinner',
  standalone: true,
  templateUrl: './loading-spinner.component.html',
  styleUrls: ['./loading-spinner.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LoadingSpinnerComponent {
  /**
   * The color using CSS var.
   *
   * Default is var(--base-primary)
   */
  @Input() color: string = 'var(--base-primary)';
}
