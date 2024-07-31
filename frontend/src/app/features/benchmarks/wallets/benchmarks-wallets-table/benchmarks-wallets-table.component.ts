import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { BenchmarksWallet } from '@shared/types/benchmarks/wallets/benchmarks-wallet.type';
import { filter } from 'rxjs';
import { selectBenchmarksWallets } from '@benchmarks/wallets/benchmarks-wallets.state';
import { TableColumnList } from '@openmina/shared';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-benchmarks-wallets-table',
  templateUrl: './benchmarks-wallets-table.component.html',
  styleUrls: ['./benchmarks-wallets-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class BenchmarksWalletsTableComponent extends MinaTableRustWrapper<BenchmarksWallet> implements OnInit {

  protected readonly tableHeads: TableColumnList<BenchmarksWallet> = [
    { name: 'public key' },
    { name: 'balance' },
    { name: 'nonce' },
    { name: 'last tx. time' },
    { name: 'last tx. memo' },
    { name: 'last tx. status' },
    { name: 'txs. ratio' },
  ];

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToWalletChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [220, 100, 100, 170, 140, 125, 160];
  }

  private listenToWalletChanges(): void {
    this.select(selectBenchmarksWallets, (wallets: BenchmarksWallet[]) => {
      this.table.rows = wallets;
      this.table.detect();
    }, filter(wallets => wallets.length > 0));
  }
}

