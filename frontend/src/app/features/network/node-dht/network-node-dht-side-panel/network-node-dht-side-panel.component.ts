import { ChangeDetectionStrategy, Component, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { selectNetworkNodeDhtActivePeer } from '@network/node-dht/network-node-dht.state';
import { NetworkNodeDhtSetActivePeer } from '@network/node-dht/network-node-dht.actions';
import { downloadJson, ExpandTracking, MinaJsonViewerComponent } from '@openmina/shared';
import { delay, mergeMap, of } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { Router } from '@angular/router';
import { isSubFeatureEnabled } from '@shared/constants/config';
import { AppSelectors } from '@app/app.state';

@Component({
  selector: 'mina-network-node-dht-side-panel',
  templateUrl: './network-node-dht-side-panel.component.html',
  styleUrls: ['./network-node-dht-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NetworkNodeDhtSidePanelComponent extends StoreDispatcher implements OnInit {

  activePeer: NetworkNodeDhtPeer;
  expandingTracking: ExpandTracking = {};
  jsonString: string;
  hasBootstrapStatsEnabled: boolean;

  @ViewChild(MinaJsonViewerComponent) private minaJsonViewer: MinaJsonViewerComponent;

  constructor(private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToActiveNode();
    this.listenToActivePeer();
  }

  private listenToActivePeer(): void {
    this.select(selectNetworkNodeDhtActivePeer, (peer: NetworkNodeDhtPeer) => {
      this.activePeer = peer;
      this.jsonString = JSON.stringify(peer);
      this.detect();
    });
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, (node) => {
      this.hasBootstrapStatsEnabled = isSubFeatureEnabled(node, 'network', 'bootstrap-stats');
    });
  }

  downloadJson(): void {
    downloadJson(this.jsonString, 'dht-peer.json');
  }

  expandEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(true);
  }

  collapseEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(false);
  }

  closeSidePanel(): void {
    this.dispatch(NetworkNodeDhtSetActivePeer, undefined);
    this.router.navigate([Routes.NETWORK, Routes.NODE_DHT], { queryParamsHandling: 'merge' });
  }

  goToBootstrapStats(): void {
    this.router.navigate([Routes.NETWORK, Routes.BOOTSTRAP_STATS, this.activePeer.peerId], { queryParamsHandling: 'merge' });
  }
}
