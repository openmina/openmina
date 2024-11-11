import { ChangeDetectionStrategy, Component, Input } from '@angular/core';

@Component({
  selector: 'mina-web-node-not-supported',
  standalone: true,
  imports: [],
  templateUrl: './web-node-not-supported.component.html',
  styleUrl: './web-node-not-supported.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 flex-column align-center' },
})
export class WebNodeNotSupportedComponent {
  @Input() isPhone!: boolean;
}
