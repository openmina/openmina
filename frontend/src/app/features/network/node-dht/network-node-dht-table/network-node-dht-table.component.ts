import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { getMergedRoute, MergedRoute, TableColumnList } from '@openmina/shared';
import { Router } from '@angular/router';
import {
  NetworkNodeDhtPeer,
  NetworkNodeDhtPeerConnectionType,
} from '@shared/types/network/node-dht/network-node-dht.type';
import { selectNetworkNodeDhtActivePeer, selectNetworkNodeDhtPeers } from '@network/node-dht/network-node-dht.state';
import { NetworkNodeDhtSetActivePeer } from '@network/node-dht/network-node-dht.actions';
import { filter, take } from 'rxjs';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';
import {
  NetworkBootstrapStatsSetActiveBootstrapRequest,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';
import { Routes } from '@shared/enums/routes.enum';


@Component({
  selector: 'mina-network-node-dht-table',
  templateUrl: './network-node-dht-table.component.html',
  styleUrls: ['./network-node-dht-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 w-100 flex-1' },
})
export class NetworkNodeDhtTableComponent extends MinaTableRustWrapper<NetworkNodeDhtPeer> implements OnInit {

  protected readonly NetworkNodeDhtPeerConnectionType = NetworkNodeDhtPeerConnectionType;
  protected readonly tableHeads: TableColumnList<NetworkNodeDhtPeer> = [
    { name: 'connection' },
    { name: 'peerId' },
    { name: 'addr. count' },
    { name: 'HEX distance' },
    { name: 'binary distance' },
    { name: 'Zero prefixes' },
    { name: 'bucket' },
  ];

  rows: NetworkNodeDhtPeer[] = [];
  activeRow: NetworkNodeDhtPeer;

  private idFromRoute: string;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToNetworkConnectionsChanges();
    this.listenToActiveRowChange();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [140, 140, 110, 130, 160, 120, 110];
    this.table.propertyForActiveCheck = 'peerId';
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['id'] && this.table.rows.length === 0) {
        this.idFromRoute = route.params['id'];
      }
    }, take(1));
  }

  private listenToNetworkConnectionsChanges(): void {
    this.select(selectNetworkNodeDhtPeers, (rows: NetworkNodeDhtPeer[]) => {
      this.rows = rows;
      this.table.rows = rows;
      this.table.detect();
      this.scrollToElement();
    }, filter(rows => rows.length > 0));
  }

  private listenToActiveRowChange(): void {
    this.select(selectNetworkNodeDhtActivePeer, (activeRow: NetworkNodeDhtPeer) => {
      this.activeRow = activeRow;
      this.table.activeRow = activeRow;
      this.table.detect();
    });
  }

  private scrollToElement(): void {
    if (this.idFromRoute) {
      this.table.scrollToElement(request => request.peerId === this.idFromRoute);
      this.setActiveRow(this.table.rows.find(request => request.peerId === this.idFromRoute));
      delete this.idFromRoute;
    }
  }

  protected override onRowClick(row: NetworkNodeDhtPeer): void {
    if (this.table.activeRow?.peerId !== row?.peerId) {
      this.setActiveRow(row);
      this.router.navigate([Routes.NETWORK, Routes.NODE_DHT, row.peerId], { queryParamsHandling: 'merge' });
    }
  }

  private setActiveRow(row: NetworkNodeDhtPeer): void {
    this.dispatch(NetworkNodeDhtSetActivePeer, row);
  }
}
