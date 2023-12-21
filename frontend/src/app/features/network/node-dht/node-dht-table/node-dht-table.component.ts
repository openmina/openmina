import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { TableColumnList } from '@openmina/shared';
import { Router } from '@angular/router';

export interface NetworkNodeDHT {
  peerStatus: string;
  id: string;
  address: string;
  lastUpdate: string;
  xorDistance: string;
  bucketRange: string;
  bucketCapacity: string;
}

@Component({
  selector: 'app-node-dht-table',
  templateUrl: './node-dht-table.component.html',
  styleUrls: ['./node-dht-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class NodeDhtTableComponent extends MinaTableRustWrapper<NetworkNodeDHT> implements OnInit {

  protected readonly tableHeads: TableColumnList<NetworkNodeDHT> = [
    { name: 'peer status' },
    { name: 'id' },
    { name: 'address' },
    { name: 'last update' },
    { name: 'XOR distance' },
    { name: 'bucket range' },
    { name: 'bucket capacity' },
  ];

  rows: NetworkNodeDHT[] = [];
  activeRow: NetworkNodeDHT;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToNetworkConnectionsChanges();
    this.listenToActiveRowChange();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [140, 110, 165, 110, 100, 120, 110];
    this.table.propertyForActiveCheck = 'id';
  }

  private listenToNetworkConnectionsChanges(): void {
    const rows: NetworkNodeDHT[] = [
      //@formatter:off
      { peerStatus: 'Connected', id: '10101001011', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '8', bucketRange: '1-2', bucketCapacity: '3' },
      { peerStatus: 'Connected', id: '010101010101', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '8', bucketRange: '1-2', bucketCapacity: '5' },
      { peerStatus: 'Connected', id: '0100000100', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '8', bucketRange: '1-2', bucketCapacity: '4' },
      { peerStatus: 'Connected', id: '1010101111', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '1', bucketRange: '1-2', bucketCapacity: '4' },
      { peerStatus: 'Discovered', id: '1110110101', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '7', bucketRange: '1-2', bucketCapacity: '4' },
      { peerStatus: 'Connected', id: '0010100000', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '8', bucketRange: '1-2', bucketCapacity: '2' },
      { peerStatus: 'Connected', id: '0101000010', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '8', bucketRange: '1-2', bucketCapacity: '4' },
      { peerStatus: 'Connected', id: '101010101010', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '8', bucketRange: '1-2', bucketCapacity: '1' },
      { peerStatus: 'Discovered', id: '0011010101', address: '2432.53.36.45:8302', lastUpdate: '2m ago', xorDistance: '10', bucketRange: '1-2', bucketCapacity: '4' },
      { peerStatus: 'Discovered', id: '11111010101', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '2', bucketRange: '1-3', bucketCapacity: '7' },
      { peerStatus: 'Connected', id: '111010111', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '2', bucketRange: '1-3', bucketCapacity: '7' },
      { peerStatus: 'Discovered', id: '0001010100', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '2', bucketRange: '1-3', bucketCapacity: '7' },
      { peerStatus: 'Connected', id: '01000010101', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '2', bucketRange: '1-3', bucketCapacity: '7' },
      { peerStatus: 'Connected', id: '001010001', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '2', bucketRange: '1-3', bucketCapacity: '7' },
      { peerStatus: 'Discovered', id: '0100010101', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '4', bucketRange: '1-3', bucketCapacity: '11' },
      { peerStatus: 'Connected', id: '011010101', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '5', bucketRange: '1-3', bucketCapacity: '33' },
      { peerStatus: 'Connected', id: '101010010', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '2', bucketRange: '1-3', bucketCapacity: '7' },
      { peerStatus: 'Connected', id: '00001010', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '22', bucketRange: '1-3', bucketCapacity: '9' },
      { peerStatus: 'Discovered', id: '0101001', address: '2432.24.36.45:8302', lastUpdate: '4m ago', xorDistance: '2', bucketRange: '1-3', bucketCapacity: '6' },
      { peerStatus: 'Discovered', id: '1010011001', address: '2432.25.36.45:8302', lastUpdate: '4m ago', xorDistance: '1', bucketRange: '1-3', bucketCapacity: '8' },
    ];
    this.rows = rows;
    this.table.rows = rows;
    this.table.detect();
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
