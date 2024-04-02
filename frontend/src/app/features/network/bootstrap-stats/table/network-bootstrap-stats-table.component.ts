import { ChangeDetectionStrategy, Component } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { getMergedRoute, MergedRoute, SecDurationConfig, TableColumnList } from '@openmina/shared';
import { filter, take } from 'rxjs';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';
import {
  selectNetworkBootstrapStatsActiveBootstrapRequest,
  selectNetworkBootstrapStatsList,
  selectNetworkBootstrapStatsSort,
} from '@network/bootstrap-stats/network-bootstrap-stats.state';
import {
  NetworkBootstrapStatsSetActiveBootstrapRequest,
  NetworkBootstrapStatsSort,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';
import { Routes } from '@shared/enums/routes.enum';
import { Router } from '@angular/router';

@Component({
  selector: 'mina-network-bootstrap-stats-table',
  templateUrl: './network-bootstrap-stats-table.component.html',
  styleUrls: ['./network-bootstrap-stats-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NetworkBootstrapStatsTableComponent extends MinaTableRustWrapper<NetworkBootstrapStatsRequest> {

  protected readonly config: SecDurationConfig = {
    includeMinutes: true,
    includeHours: true,
    undefinedAlternative: '-',
  };
  protected readonly tableHeads: TableColumnList<NetworkBootstrapStatsRequest> = [
    { name: 'datetime', sort: 'start' },
    { name: 'result', sort: 'type' },
    { name: 'duration' },
    { name: 'peerId' },
    { name: 'address' },
    { name: 'existing peers', sort: 'existingPeers' },
    { name: 'new peers', sort: 'newPeers' },
  ];
  private idFromRoute: string;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToRequestsChanges();
    this.listenToActiveRequestChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [165, 150, 100, 150, 250, 120, 100];
    this.table.minWidth = 1035;
    this.table.sortClz = NetworkBootstrapStatsSort;
    this.table.sortSelector = selectNetworkBootstrapStatsSort;
    this.table.propertyForActiveCheck = 'peerId';
    this.table.trackByFn = (_: number, row: NetworkBootstrapStatsRequest) => row.peerId + row.type + row.existingPeers + row.newPeers + row.finish;
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['id'] && this.table.rows.length === 0) {
        this.idFromRoute = route.params['id'];
      }
    }, take(1));
  }

  private listenToRequestsChanges(): void {
    this.select(selectNetworkBootstrapStatsList, (requests: NetworkBootstrapStatsRequest[]) => {
      this.table.rows = requests;
      this.table.detect();
      this.scrollToElement();
    }, filter(requests => requests.length > 0));
  }

  private listenToActiveRequestChanges(): void {
    this.select(selectNetworkBootstrapStatsActiveBootstrapRequest, (request: NetworkBootstrapStatsRequest) => {
      this.table.activeRow = request;
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

  protected override onRowClick(row: NetworkBootstrapStatsRequest): void {
    if (this.table.activeRow?.peerId !== row?.peerId) {
      this.setActiveRow(row);
      this.router.navigate([Routes.NETWORK, Routes.BOOTSTRAP_STATS, row.peerId], { queryParamsHandling: 'merge' });
    }
  }

  private setActiveRow(row: NetworkBootstrapStatsRequest): void {
    this.dispatch(NetworkBootstrapStatsSetActiveBootstrapRequest, row);
  }
}
