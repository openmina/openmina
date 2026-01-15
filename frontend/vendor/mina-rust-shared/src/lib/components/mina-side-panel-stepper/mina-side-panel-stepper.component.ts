import { ChangeDetectionStrategy, Component, Input, TemplateRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MinaRustSharedModule } from '../../mina-rust-shared.module';
import { REQUIRED } from '../../constants/angular';


@Component({
    selector: 'mina-side-panel-stepper',
    templateUrl: './mina-side-panel-stepper.component.html',
    styleUrls: ['./mina-side-panel-stepper.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    imports: [CommonModule, MinaRustSharedModule],
    host: { class: 'w-100 h-100 p-relative overflow-hidden flex-column' }
})
export class MinaSidePanelStepperComponent {
  /**
   * @description zero-based index of the active step
   */
  @Input(REQUIRED) activeStep: number = 0;
  @Input(REQUIRED) steps: TemplateRef<void>[];
}
