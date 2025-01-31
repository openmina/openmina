import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { Platform } from '@angular/cdk/platform';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { getMergedRoute, MergedRoute } from '@openmina/shared';
import { filter } from 'rxjs';
import { WebNodeService } from '@core/services/web-node.service';
import { iOSversion } from '@shared/helpers/webnode.helper';
import { WebNodeInitializationComponent } from '@web-node/web-node-initialization/web-node-initialization.component';
import { WebNodeNotSupportedComponent } from '@web-node/web-node-not-supported/web-node-not-supported.component';
import { WebNodeFileUploadComponent } from '@web-node/web-node-file-upload/web-node-file-upload.component';

@Component({
  selector: 'mina-web-node',
  standalone: true,
  imports: [
    WebNodeInitializationComponent,
    WebNodeNotSupportedComponent,
    WebNodeFileUploadComponent,
  ],
  templateUrl: './web-node.component.html',
  styleUrl: './web-node.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WebNodeComponent extends StoreDispatcher implements OnInit {

  supported: boolean = false;
  isPhone: boolean = false;
  showFileUpload: boolean = true;

  constructor(private platform: Platform,
              private webNodeService: WebNodeService) { super(); }

  ngOnInit(): void {
    document.body.style.backgroundColor = 'var(--base-background)';
    this.listenToFileUploadingEvents();
    this.checkIfDeviceIsSupported();
    this.listenToRoute();
  }

  private listenToFileUploadingEvents(): void {
    // this.showFileUpload = false;
  }

  private listenToRoute(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      let initial = 176;
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
      this.webNodeService.memory = { initial, maximum, shared };
    }, filter(Boolean));
  }

  private checkIfDeviceIsSupported(): void {
    if (this.platform.IOS) {
      this.supported = iOSversion()[0] >= 18;
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
