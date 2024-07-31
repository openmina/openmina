import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { isMobile } from '@openmina/shared';
import { MempoolSelectors } from '@app/features/mempool/mempool.state';
import { MempoolFilters } from '@shared/types/mempool/mempool-filters.type';
import { MempoolTransactionKind } from '@shared/types/mempool/mempool-transaction.type';
import { MempoolActions } from '@app/features/mempool/mempool.actions';
import { FormBuilder, FormControl, FormGroup } from '@angular/forms';
import { untilDestroyed } from '@ngneat/until-destroy';
import { debounceTime, distinctUntilChanged, filter } from 'rxjs';

@Component({
  selector: 'mina-mempool-filters',
  templateUrl: './mempool-filters.component.html',
  styleUrls: ['./mempool-filters.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MempoolFiltersComponent extends StoreDispatcher implements OnInit {

  protected readonly isMobile = isMobile();
  protected readonly formGroup: FormGroup<{ search: FormControl<string> }> = this.fb.group({ search: [''] });

  filters: MempoolFilters;
  zkApps: number = 0;
  payments: number = 0;
  delegations: number = 0;

  constructor(private fb: FormBuilder) { super(); }

  ngOnInit(): void {
    this.listenToFilters();
    this.listenToActiveEpoch();
    this.listenToSearchChanges();
  }

  private listenToSearchChanges(): void {
    this.formGroup.get('search').valueChanges.pipe(
      distinctUntilChanged(),
      debounceTime(200),
      filter((value: string) => {
        if (value.length <= 2) {
          this.changeFilter('search', value.trim());
          return false;
        }
        return true;
      }),
      untilDestroyed(this),
    ).subscribe((value: string) => {
      this.changeFilter('search', value.trim());
    });
  }

  private listenToFilters(): void {
    this.select(MempoolSelectors.filters, filters => {
      this.filters = filters;
      const search = this.formGroup.get('search');
      if (filters.search !== search.value) {
        search.setValue(filters.search);
      }
      this.detect();
    });
  }

  private listenToActiveEpoch(): void {
    this.select(MempoolSelectors.allTxs, txs => {
      this.zkApps = txs.filter(tx => tx.kind === MempoolTransactionKind.ZK_APP).length;
      this.payments = txs.filter(tx => tx.kind === MempoolTransactionKind.PAYMENT).length;
      this.delegations = txs.filter(tx => tx.kind === MempoolTransactionKind.DELEGATION).length;
      this.detect();
    });
  }

  changeFilter(filter: keyof MempoolFilters, value: boolean | string): void {
    this.dispatch2(MempoolActions.changeFilters({ filters: { ...this.filters, [filter]: value } }));
  }

}
