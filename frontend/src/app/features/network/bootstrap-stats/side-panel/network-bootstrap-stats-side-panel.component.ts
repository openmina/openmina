import { ChangeDetectionStrategy, Component, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import {
  selectNetworkNodeDhtActiveBootstrapRequest,
  selectNetworkNodeDhtActivePeer,
} from '@network/node-dht/network-node-dht.state';
import { NetworkNodeDhtSetActivePeer, NetworkNodeDhtToggleSidePanel } from '@network/node-dht/network-node-dht.actions';
import { ExpandTracking, MinaJsonViewerComponent, downloadJson } from '@openmina/shared';
import { delay, mergeMap, of } from 'rxjs';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';
import {
  selectNetworkBootstrapStatsActiveBootstrapRequest,
} from '@network/bootstrap-stats/network-bootstrap-stats.state';
import {
  NetworkBootstrapStatsSetActiveBootstrapRequest,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';
import { Routes } from '@shared/enums/routes.enum';
import { Router } from '@angular/router';

@Component({
  selector: 'mina-network-bootstrap-stats-side-panel',
  templateUrl: './network-bootstrap-stats-side-panel.component.html',
  styleUrls: ['./network-bootstrap-stats-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NetworkBootstrapStatsSidePanelComponent extends StoreDispatcher implements OnInit {

  request: NetworkBootstrapStatsRequest;
  expandingTracking: ExpandTracking = {};
  jsonString: string;
  activeTab: number = 1;

  @ViewChild(MinaJsonViewerComponent) private minaJsonViewer: MinaJsonViewerComponent;

  constructor(private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(selectNetworkBootstrapStatsActiveBootstrapRequest, (request: NetworkBootstrapStatsRequest) => {
      this.request = { ...request };
      if (!this.request.error) {
        delete this.request.error;
      }
      if (!this.request.finish) {
        delete this.request.finish;
        delete this.request.duration;
      }
      this.jsonString = JSON.stringify(request);
      this.detect();
    });
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

  closeSidePanel(): void {
    this.router.navigate([Routes.NETWORK, Routes.BOOTSTRAP_STATS], { queryParamsHandling: 'merge' });
    this.dispatch(NetworkBootstrapStatsSetActiveBootstrapRequest, undefined);
  }

  selectTab(number: number): void {
    this.activeTab = number;
  }
}
