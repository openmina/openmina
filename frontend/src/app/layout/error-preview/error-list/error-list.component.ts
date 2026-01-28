import {
  ChangeDetectionStrategy,
  Component,
  EventEmitter,
  Input,
  Output,
} from '@angular/core';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { MinaError } from '@shared/types/error-preview/mina-error.type';
import { ManualDetection, MinaRustEagerSharedModule } from '@mina-rust/shared';
import { NgClass } from '@angular/common';

@Component({
  selector: 'mina-error-list',
  templateUrl: './error-list.component.html',
  styleUrls: ['./error-list.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'border-rad-6 border overflow-y-auto' },
  standalone: true,
  imports: [NgClass, MinaRustEagerSharedModule],
})
export class ErrorListComponent extends ManualDetection {
  readonly errorIconMap: any = {
    [MinaErrorType.RUST]: 'terminal',
    [MinaErrorType.GENERIC]: 'error',
    [MinaErrorType.DEBUGGER]: 'code',
  };

  @Input() errors: MinaError[];
  @Output() onConfirm: EventEmitter<any> = new EventEmitter<any>();

  constructor() {
    super();
  }

  close(): void {
    this.onConfirm.emit();
  }
}
