import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';
import {
  selectNetworkNodeDhtActiveBootstrapRequest,
  selectNetworkNodeDhtActivePeer,
} from '@network/node-dht/network-node-dht.state';
import { NetworkNodeDhtSetActivePeer, NetworkNodeDhtToggleSidePanel } from '@network/node-dht/network-node-dht.actions';

@Component({
  selector: 'mina-network-node-dht-side-panel',
  templateUrl: './network-node-dht-side-panel.component.html',
  styleUrls: ['./network-node-dht-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NetworkNodeDhtSidePanelComponent extends StoreDispatcher implements OnInit {

  activePeer: NetworkNodeDHT;
  activeStep: number = 0;

  ngOnInit(): void {
    this.listenToActiveNode();
    this.listenToActiveBoostrapRequest();
  }

  private listenToActiveNode(): void {
    this.select(selectNetworkNodeDhtActivePeer, (peer: NetworkNodeDHT) => {
      this.activePeer = peer;
      if (this.activePeer) {
        this.activeStep = 1;
      } else {
        this.activeStep = 0;
      }
      this.detect();
    });
  }

  private listenToActiveBoostrapRequest(): void {
    this.select(selectNetworkNodeDhtActiveBootstrapRequest, (request: any) => {
      if (request) {
        this.activeStep = 2;
      } else {
        this.activeStep = 0;
      }
      this.detect();
    });
  }

  removeActivePeer(): void {
    this.dispatch(NetworkNodeDhtSetActivePeer);
  }

  toggleSidePanel(): void {
    this.dispatch(NetworkNodeDhtToggleSidePanel);
  }

  backToStep0(): void {
    this.activeStep = 0;
  }
}
