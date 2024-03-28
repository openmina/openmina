import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectNetworkNodeDhtBootstrapStats } from '@network/node-dht/network-node-dht.state';
import { NetworkNodeDhtSetActiveBootstrapRequest } from '@network/node-dht/network-node-dht.actions';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

@Component({
  selector: 'mina-network-node-dht-bootstrap-stats',
  templateUrl: './network-node-dht-bootstrap-stats.component.html',
  styleUrls: ['./network-node-dht-bootstrap-stats.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkNodeDhtBootstrapStatsComponent extends StoreDispatcher implements OnInit {

  bootstrapStats: NetworkBootstrapStatsRequest[] = [];

  readonly trackStats = (_: number, stat: NetworkBootstrapStatsRequest) => stat.type;

  ngOnInit(): void {
    this.listenToBootstrapStats();
  }

  private listenToBootstrapStats(): void {
    this.select(selectNetworkNodeDhtBootstrapStats, (stats: NetworkBootstrapStatsRequest[]) => {
      this.bootstrapStats = stats;
      this.detect();
    });
  }

  setActiveBootstrapRequest(request: any): void {
    this.dispatch(NetworkNodeDhtSetActiveBootstrapRequest, request);
  }
}
