import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectNetworkNodeDhtBootstrapStats } from '@network/node-dht/network-node-dht.state';
import { NetworkNodeDhtBootstrapStats } from '@shared/types/network/node-dht/network-node-dht-bootstrap-stats.type';
import { NetworkNodeDhtSetActiveBootstrapRequest } from '@network/node-dht/network-node-dht.actions';

@Component({
  selector: 'mina-network-node-dht-bootstrap-stats',
  templateUrl: './network-node-dht-bootstrap-stats.component.html',
  styleUrls: ['./network-node-dht-bootstrap-stats.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkNodeDhtBootstrapStatsComponent extends StoreDispatcher implements OnInit {

  bootstrapStats: NetworkNodeDhtBootstrapStats[] = [];

  ngOnInit(): void {
    this.listenToBootstrapStats();
  }

  private listenToBootstrapStats(): void {
    this.select(selectNetworkNodeDhtBootstrapStats, (stats: NetworkNodeDhtBootstrapStats[]) => {
      this.bootstrapStats = stats;
      this.detect();
    });
  }

  setActiveBootstrapRequest(request: any): void {
    this.dispatch(NetworkNodeDhtSetActiveBootstrapRequest, request);
  }
}
