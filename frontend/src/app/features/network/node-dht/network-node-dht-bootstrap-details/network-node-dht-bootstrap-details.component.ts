import { ChangeDetectionStrategy, Component, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { downloadJson, ExpandTracking, MinaJsonViewerComponent } from '@openmina/shared';
import { selectNetworkNodeDhtActiveBootstrapRequest } from '@network/node-dht/network-node-dht.state';
import { delay, mergeMap, of } from 'rxjs';

@Component({
  selector: 'mina-network-node-dht-bootstrap-details',
  templateUrl: './network-node-dht-bootstrap-details.component.html',
  styleUrls: ['./network-node-dht-bootstrap-details.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkNodeDhtBootstrapDetailsComponent extends StoreDispatcher implements OnInit {

  request: any;
  expandingTracking: ExpandTracking = {};
  jsonString: string;

  @ViewChild(MinaJsonViewerComponent) private minaJsonViewer: MinaJsonViewerComponent;

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(selectNetworkNodeDhtActiveBootstrapRequest, (request: any) => {
      this.request = request;
      this.jsonString = JSON.stringify(request);
      this.detect();
    }, mergeMap((request: any) => of(request).pipe(delay(request ? 0 : 300))));
  }

  downloadJson(): void {
    downloadJson(this.jsonString, 'bootstrap-request.json');
  }

  expandEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(true);
  }

  collapseEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(false);
  }
}
