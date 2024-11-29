import { ChangeDetectionStrategy, Component, EventEmitter } from '@angular/core';
import { AppEnvBuild } from '@shared/types/app/app-env-build.type';
import { ManualDetection, MinaJsonViewerComponent } from '@openmina/shared';

@Component({
  selector: 'mina-env-build-modal',
  standalone: true,
  imports: [
    MinaJsonViewerComponent,
  ],
  templateUrl: './env-build-modal.component.html',
  styleUrl: './env-build-modal.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100 bg-surface border-rad-6 border pb-12' },
})
export class EnvBuildModalComponent extends ManualDetection {
  envBuild: AppEnvBuild;
  close: EventEmitter<void> = new EventEmitter<void>();
}
