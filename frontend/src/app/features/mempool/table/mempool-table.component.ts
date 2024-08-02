import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { getMergedRoute, MergedRoute, TableColumnList } from '@openmina/shared';
import { Router } from '@angular/router';
import { skip, take } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { MempoolTransaction } from '@shared/types/mempool/mempool-transaction.type';
import { MempoolActions } from '@app/features/mempool/mempool.actions';
import { MempoolSelectors } from '@app/features/mempool/mempool.state';

@Component({
  selector: 'mina-mempool-table',
  templateUrl: './mempool-table.component.html',
  styleUrls: ['./mempool-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100 h-100' },
})
export class MempoolTableComponent extends MinaTableRustWrapper<MempoolTransaction> implements OnInit {

  protected readonly tableHeads: TableColumnList<MempoolTransaction> = [
    { name: 'kind' },
    { name: 'tx hash' },
    { name: 'sender' },
    { name: 'fee (MINA)' },
    { name: 'amount (MINA)' },
    { name: 'nonce' },
    { name: 'memo' },
    { name: '' },
  ];

  emptyBecauseOfFilters: boolean;
  emptyInDatabase: boolean;
  isLoading: boolean;

  private fromRoute: string;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToEmptyInDatabase();
    this.listenToRouteChange();
    this.listenToActiveTxChange();
    this.listenToTxsChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [140, 130, 130, 100, 120, 90, 180, 80];
    this.table.minWidth = 980;
    this.table.propertyForActiveCheck = 'txHash';
    this.table.trackByFn = (_, tx) => tx.txHash + tx.kind + tx.nonce;
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['id'] && this.table.rows.length === 0) {
        this.fromRoute = route.params['id'];
      }
    }, take(1));
  }

  private listenToEmptyInDatabase(): void {
    this.select(MempoolSelectors.emptyInDatabase, (empty: boolean) => {
      this.emptyInDatabase = empty;
    });
    this.select(MempoolSelectors.isLoading, (loading: boolean) => {
      this.isLoading = loading;
      this.detect();
    });
  }

  private listenToTxsChanges(): void {
    this.select(MempoolSelectors.filteredTxs, (txs: MempoolTransaction[]) => {
      this.table.rows = txs;
      this.emptyBecauseOfFilters = txs.length === 0;
      this.table.detect();
      if (this.fromRoute && txs.length > 0) {
        this.scrollToElement();
      }
      this.detect();
    });
  }

  private listenToActiveTxChange(): void {
    this.select(MempoolSelectors.activeTx, (tx: MempoolTransaction) => {
      if (!this.table.activeRow) {
        this.fromRoute = tx?.txHash;
      }
      this.table.activeRow = tx;
      this.table.detect();
      this.detect();
    }, skip(1));
  }

  private scrollToElement(): void {
    const finder = (node: MempoolTransaction) => node.txHash === this.fromRoute;
    const i = this.table.rows.findIndex(finder);
    this.table.scrollToElement(finder);
    delete this.fromRoute;
    this.onRowClick(this.table.rows[i]);
  }

  protected override onRowClick(tx: MempoolTransaction): void {
    if (!tx) {
      return;
    }
    if (this.table.activeRow?.txHash !== tx?.txHash) {
      this.dispatch2(MempoolActions.setActiveTx({ tx }));
      this.router.navigate([Routes.MEMPOOL, tx.txHash], { queryParamsHandling: 'merge' });
    }
  }

  clearFilters(): void {
    this.dispatch2(MempoolActions.changeFilters({
      filters: {
        search: '',
        zkApp: true,
        payment: true,
        delegation: true,
      },
    }));
  }
}
