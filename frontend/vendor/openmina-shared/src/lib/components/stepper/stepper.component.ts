import { ChangeDetectionStrategy, Component, Input, TemplateRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { OpenminaEagerSharedModule } from '../../openmina-eager-shared.module';

@Component({
    imports: [OpenminaEagerSharedModule, CommonModule],
    selector: 'mina-stepper',
    templateUrl: './stepper.component.html',
    styleUrls: ['./stepper.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    host: { class: 'w-100 mt-16 flex-column' }
})
export class StepperComponent {

  @Input() steps: TemplateRef<any>[];
  /**
   * @description zero-based index of the active step
   */
  @Input() activeStep: number = 0;
  @Input() stepHeaders: string[];
  @Input() contentHeaderInfoTemplate: TemplateRef<any>;
  @Input() footerTemplate: TemplateRef<any>;
  @Input() height: number = 280;

}
