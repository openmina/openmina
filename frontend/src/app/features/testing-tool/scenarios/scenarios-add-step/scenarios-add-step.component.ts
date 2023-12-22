import { ChangeDetectionStrategy, Component, EventEmitter, Input, Output } from '@angular/core';

@Component({
  selector: 'mina-scenarios-add-step',
  templateUrl: './scenarios-add-step.component.html',
  styleUrls: ['./scenarios-add-step.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ScenariosAddStepComponent {

  @Input() currentStep: number;

  @Output() cancel = new EventEmitter<void>();
  @Output() confirm = new EventEmitter<string>();

  pastedText: string;
}
