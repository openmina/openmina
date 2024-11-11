import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { WebNodeInitializationComponent } from '@app/features/webnode/web-node-initialization/web-node-initialization.component';
import { Platform } from '@angular/cdk/platform';
import { WebNodeNotSupportedComponent } from '@app/features/webnode/web-node-not-supported/web-node-not-supported.component';

@Component({
  selector: 'mina-webnode',
  standalone: true,
  imports: [
    WebNodeInitializationComponent,
    WebNodeNotSupportedComponent,
  ],
  templateUrl: './webnode.component.html',
  styleUrl: './webnode.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WebnodeComponent implements OnInit {

  supported: boolean = false;
  isPhone: boolean = false;

  constructor(private platform: Platform) {}

  ngOnInit(): void {
    this.checkIfDeviceIsSupported();
  }

  private checkIfDeviceIsSupported(): void {
    if (this.platform.IOS) {
      this.supported = false;
      this.isPhone = true;
      return;
    }

    if (this.platform.FIREFOX) {
      this.supported = false;
      return;
    }

    this.supported = true;
  }
}
