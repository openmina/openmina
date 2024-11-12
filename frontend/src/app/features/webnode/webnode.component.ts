import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { WebNodeInitializationComponent } from '@app/features/webnode/web-node-initialization/web-node-initialization.component';
import { Platform } from '@angular/cdk/platform';
import { WebNodeNotSupportedComponent } from '@app/features/webnode/web-node-not-supported/web-node-not-supported.component';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { getMergedRoute, MergedRoute } from '@openmina/shared';
import { filter } from 'rxjs';
import { WebNodeService } from '@core/services/web-node.service';

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
export class WebnodeComponent extends StoreDispatcher implements OnInit {

  supported: boolean = false;
  isPhone: boolean = false;

  constructor(private platform: Platform,
              private webNodeService: WebNodeService) { super(); }

  ngOnInit(): void {
    this.checkIfDeviceIsSupported();
    this.listenToRoute();
  }

  private listenToRoute(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      let initial = 174;
      if (route.queryParams['initial']) {
        initial = Number(route.queryParams['initial']);
      }
      let maximum = 65536;
      if (route.queryParams['maximum']) {
        maximum = Number(route.queryParams['maximum']);
      }
      let shared = true;
      if (route.queryParams['shared']) {
        shared = route.queryParams['shared'] === 'true';
      }
      this.webNodeService.memory = new WebAssembly.Memory({ initial, maximum, shared });
    }, filter(Boolean));
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
