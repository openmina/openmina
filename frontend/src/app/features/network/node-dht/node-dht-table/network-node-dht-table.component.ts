import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { TableColumnList } from '@openmina/shared';
import { Router } from '@angular/router';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';
import { selectNetworkNodeDhtPeers } from '@network/node-dht/network-node-dht.state';


@Component({
  selector: 'mina-network-node-dht-table',
  templateUrl: './network-node-dht-table.component.html',
  styleUrls: ['./network-node-dht-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NetworkNodeDhtTableComponent extends MinaTableRustWrapper<NetworkNodeDHT> implements OnInit {

  protected readonly tableHeads: TableColumnList<NetworkNodeDHT> = [
    { name: 'peerId' },
    { name: 'addresses' },
    { name: 'HEX distance' },
    { name: 'binary distance' },
    { name: 'XOR distance' },
    { name: 'bucket index' },
  ];

  rows: NetworkNodeDHT[] = [];
  activeRow: NetworkNodeDHT;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToNetworkConnectionsChanges();
    // this.listenToActiveRowChange();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [140, 110, 130, 160, 120, 110];
    this.table.propertyForActiveCheck = 'peerId';
  }

  private listenToNetworkConnectionsChanges(): void {
    this.select(selectNetworkNodeDhtPeers, (rows: NetworkNodeDHT[]) => {
      this.rows = rows;
      this.table.rows = rows;
      this.table.detect();
    });
  }

  private listenToActiveRowChange(): void {
    this.activeRow = this.rows[4];
    this.table.activeRow = this.activeRow;
  }

  protected override onRowClick(row: NetworkNodeDHT): void {
    this.activeRow = row;
    this.table.activeRow = this.activeRow;
  }
}
