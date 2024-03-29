import { ChangeDetectionStrategy, Component, Input } from '@angular/core';

@Component({
  selector: 'mina-card',
  templateUrl: './mina-card.component.html',
  styleUrls: ['./mina-card.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column flex-between pt-8 pb-8 pl-12 border-rad-8 bg-surface' },
})
export class MinaCardComponent {

  @Input() color: string = 'var(--base-primary)';
  @Input() labelColor: string = 'var(--base-primary)';
  @Input() hintColor: string = 'var(--base-tertiary)';
  @Input() icon: string;
  @Input() label: string | number;
  @Input() value: string | number;
  @Input() hint: string | number;
  @Input() tooltipText: string;

}
