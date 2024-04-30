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
  /**
   * The size of the spinner in px.
   *
   * Default is 16px.
   */
  @Input() size: number = 16;
  /**
   * The width of the border in px.
   *
   * Default is 1px.
   */
  @Input() borderWidth: number = 1;
}
