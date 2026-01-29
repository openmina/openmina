import {
  ChangeDetectionStrategy,
  Component,
  EventEmitter,
  Input,
  Output,
} from '@angular/core';
import { iOSversion } from '@shared/helpers/webnode.helper';
import { safelyExecuteInBrowser } from '@mina-rust/shared';

const code = [1, 2, 3, 2];

@Component({
  selector: 'mina-web-node-not-supported',
  imports: [],
  templateUrl: './web-node-not-supported.component.html',
  styleUrl: './web-node-not-supported.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 flex-column align-center' },
})
export class WebNodeNotSupportedComponent {
  @Input() isPhone!: boolean;

  @Output() bypassUnsupportedDevice = new EventEmitter<void>();

  iOSVersion: string = iOSversion().join('.');
  devMode: boolean = false;
  private codeVerifier: number[] = [];

  addDevKey2(): void {
    this.codeVerifier.push(code[this.codeVerifier.length]);
    this.checkCode();
  }

  addDevKey(key: number): void {
    this.codeVerifier.push(key);
    this.checkCode();
  }

  private checkCode(): void {
    if (this.codeVerifier.length === code.length) {
      if (this.codeVerifier.every((v, i) => v === code[i])) {
        this.devMode = true;
      } else {
        this.codeVerifier = [];
      }
    }
  }

  c_hcK1_V_a_l_id(input: HTMLInputElement): void {
    if (input.value === 'allowme') {
      this.bypassUnsupportedDevice.emit();
    }
  }

  howToUpdate(): void {
    safelyExecuteInBrowser(() =>
      window.open('https://support.apple.com/en-us/118575', '_blank'),
    );
  }
}
