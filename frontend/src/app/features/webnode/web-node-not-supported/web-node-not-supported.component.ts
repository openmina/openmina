import { ChangeDetectionStrategy, Component, EventEmitter, Input, Output } from '@angular/core';
import { Platform } from '@angular/cdk/platform';
import { sendSentryEvent } from '@shared/helpers/webnode.helper';

const code = [1, 2, 3, 2];

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

  @Output() bypassUnsupportedDevice = new EventEmitter<void>();

  devMode: boolean = false;
  private codeVerifier: number[] = [];

  constructor(private platform: Platform) {}

  addDevKey(key: number): void {
    this.codeVerifier.push(key);
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
      sendSentryEvent('A developer is testing the app on ' + this.platform, 'debug');
      this.bypassUnsupportedDevice.emit();
    }
  }
}

function iOSversion(): number {
  if (/iP(hone|od|ad)/.test(navigator.platform)) {
    // supports iOS 2.0 and later: <http://bit.ly/TJjs1V>
    const v = (navigator.appVersion).match(/OS (\d+)_(\d+)_?(\d+)?/);
    return [parseInt(v[1], 10), parseInt(v[2], 10), parseInt(v[3] || '0', 10)][0];
  }
  return 0;
}
