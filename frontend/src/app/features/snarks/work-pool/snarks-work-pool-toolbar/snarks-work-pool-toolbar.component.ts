import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectSnarksWorkPoolFilters, selectSnarksWorkPools } from '@snarks/work-pool/snarks-work-pool.state';
import { SnarksWorkPoolToggleFilter } from '@snarks/work-pool/snarks-work-pool.actions';
import { WorkPool } from '@shared/types/snarks/work-pool/work-pool.type';

@Component({
  selector: 'mina-snarks-work-pool-toolbar',
  templateUrl: './snarks-work-pool-toolbar.component.html',
  styleUrls: ['./snarks-work-pool-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'fx-row-vert-cent h-lg border-bottom' },
})
export class SnarksWorkPoolToolbarComponent extends StoreDispatcher implements OnInit {

  readonly allFilters: string[] = [
    'local',
    'remote',
  ];
  activeFilters: string[] = [];
  total: number;
  available: number;
  committed: number;
  snarked: number;

  ngOnInit(): void {
    this.listenToActiveFiltersChanges();
    this.listenToWorkPool();
  }

  private listenToActiveFiltersChanges(): void {
    this.select(selectSnarksWorkPoolFilters, (filters: string[]) => {
      this.activeFilters = filters;
      this.detect();
    });
  }

  toggleFilter(filter: string): void {
    this.dispatch(SnarksWorkPoolToggleFilter, filter);
  }

  private listenToWorkPool(): void {
    this.select(selectSnarksWorkPools, (wp: WorkPool[]) => {
      this.total = wp.length;
      this.committed = wp.filter((w: WorkPool) => w.commitment).length;
      this.snarked = wp.filter((w: WorkPool) => w.snark).length;
      this.available = wp.filter((w: WorkPool) => !w.commitment && !w.snark).length;
      this.detect();
    });

  }
}
