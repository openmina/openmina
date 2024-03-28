import { ChangeDetectionStrategy, Component } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { DashboardPeer, DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { TableColumnList } from '@openmina/shared';
import { DashboardPeersSort } from '@dashboard/dashboard.actions';
import { selectDashboardPeers, selectDashboardPeersSort } from '@dashboard/dashboard.state';
import { filter } from 'rxjs';

@Component({
  selector: 'mina-dashboard-peers-minimal-table',
  templateUrl: './dashboard-peers-minimal-table.component.html',
  styleUrls: ['./dashboard-peers-minimal-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class DashboardPeersMinimalTableComponent extends MinaTableRustWrapper<DashboardPeer> {

  protected readonly DashboardPeerStatus = DashboardPeerStatus;
  protected readonly tableHeads: TableColumnList<DashboardPeer> = [
    { name: 'peer', sort: 'status' },
    { name: 'datetime', sort: 'timestamp' },
    { name: 'requests' },
  ];

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToPeersChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [130, 165, 89];
    this.table.minWidth = 384;
    this.table.sortClz = DashboardPeersSort;
    this.table.sortSelector = selectDashboardPeersSort;
    this.table.trackByFn = (_: number, row: DashboardPeer) => row.peerId + row.status + row.bestTip + row.timestamp;
  }

  private listenToPeersChanges(): void {
    this.select(selectDashboardPeers, (peers: DashboardPeer[]) => {
      this.table.rows = peers;
      this.table.detect();
    }, filter(peers => peers.length > 0));
  }

  protected override onRowClick(row: DashboardPeer): void {
  }
}
