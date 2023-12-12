import { ChangeDetectionStrategy, Component } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { TableColumnList } from '@openmina/shared';
import { DashboardPeer, DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { filter } from 'rxjs';
import { selectDashboardPeers, selectDashboardPeersSort } from '@dashboard/dashboard.state';
import { DashboardPeersSort } from '@dashboard/dashboard.actions';

@Component({
  selector: 'mina-dashboard-peers-table',
  templateUrl: './dashboard-peers-table.component.html',
  styleUrls: ['./dashboard-peers-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class DashboardPeersTableComponent extends MinaTableRustWrapper<DashboardPeer> {

  protected readonly DashboardPeerStatus = DashboardPeerStatus;
  protected readonly tableHeads: TableColumnList<DashboardPeer> = [
    { name: 'peer ID', sort: 'peerId' },
    { name: 'status' },
    { name: 'datetime', sort: 'timestamp' },
    { name: 'address' },
    { name: 'global slot', sort: 'globalSlot' },
    { name: 'height' },
    { name: 'best tip datetime', sort: 'bestTipTimestamp' },
    { name: 'best tip', sort: 'bestTip' },
  ];

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToPeersChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [140, 140, 165, 150, 110, 100, 165, 190];
    this.table.minWidth = 1160;
    this.table.sortClz = DashboardPeersSort;
    this.table.sortSelector = selectDashboardPeersSort;
    this.table.trackByFn = (index: number, row: DashboardPeer) => row.peerId + row.status + row.bestTip + row.timestamp;
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
