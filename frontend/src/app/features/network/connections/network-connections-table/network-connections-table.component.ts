import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { NetworkConnection } from '@shared/types/network/connections/network-connection.type';
import {
  selectNetworkConnections,
  selectNetworkConnectionsActiveConnection,
} from '@network/connections/network-connections.state';
import { NetworkConnectionsSelectConnection } from '@network/connections/network-connections.actions';
import { getMergedRoute, MergedRoute, TableColumnList } from '@openmina/shared';
import { filter, take } from 'rxjs';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-network-connections-table',
  templateUrl: './network-connections-table.component.html',
  styleUrls: ['./network-connections-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NetworkConnectionsTableComponent extends MinaTableRustWrapper<NetworkConnection> implements OnInit {

  protected readonly tableHeads: TableColumnList<NetworkConnection> = [
    { name: 'ID' },
    { name: 'datetime' },
    { name: 'remote address' },
    { name: 'PID' },
    { name: 'FD' },
    { name: 'incoming' },
    { name: 'alias' },
    { name: 'decrypted in' },
    { name: 'decrypted out' },
  ];

  private connections: NetworkConnection[] = [];
  private activeRow: NetworkConnection;
  private idFromRoute: number;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToNetworkConnectionsChanges();
    this.listenToActiveRowChange();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [80, 170, 190, 90, 60, 100, 120, 110, 110];
    this.table.propertyForActiveCheck = 'connectionId';
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['id'] && this.connections.length === 0) {
        this.idFromRoute = Number(route.params['id']);
      }
    }, take(1));
  }

  private listenToNetworkConnectionsChanges(): void {
    this.select(selectNetworkConnections, (connections: NetworkConnection[]) => {
      this.connections = connections;
      this.table.rows = connections;
      this.table.detect();
      this.scrollToElement();
    }, filter(connections => connections.length > 0));
  }

  private listenToActiveRowChange(): void {
    this.select(selectNetworkConnectionsActiveConnection, (connection: NetworkConnection) => {
      this.activeRow = connection;
      this.table.activeRow = connection;
      this.table.detect();
    });
  }

  protected override onRowClick(row: NetworkConnection): void {
    if (row.connectionId !== this.activeRow?.connectionId) {
      this.router.navigate([Routes.NETWORK, Routes.CONNECTIONS, row.connectionId], { queryParamsHandling: 'merge' });
      this.dispatch(NetworkConnectionsSelectConnection, row);
    }
  }

  private scrollToElement(): void {
    if (this.idFromRoute) {
      this.table.scrollToElement(c => c.connectionId === this.idFromRoute);
      delete this.idFromRoute;
    }
  }

  goToNetworkMessages(addr: string): void {
    this.router.navigate([Routes.NETWORK, Routes.MESSAGES], {
      queryParams: { addr },
      queryParamsHandling: 'merge',
    });
  }
}
