import { ChangeDetectionStrategy, Component } from '@angular/core';

@Component({
  selector: 'app-memory-resources',
  templateUrl: './memory-resources.component.html',
  styleUrls: ['./memory-resources.component.scss'],
  host: { class: 'flex-column h-100' },
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class MemoryResourcesComponent {

}
