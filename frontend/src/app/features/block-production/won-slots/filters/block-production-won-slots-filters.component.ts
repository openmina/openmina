import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';
import {
  BlockProductionWonSlotsFilters,
} from '@shared/types/block-production/won-slots/block-production-won-slots-filters.type';
import { BlockProductionWonSlotsSelectors } from '@block-production/won-slots/block-production-won-slots.state';
import { isMobile } from '@openmina/shared';
import {
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';

@Component({
  selector: 'mina-block-production-won-slots-filters',
  templateUrl: './block-production-won-slots-filters.component.html',
  styleUrls: ['./block-production-won-slots-filters.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-xl w-100 flex-row border-top border-bottom' },
})
export class BlockProductionWonSlotsFiltersComponent extends StoreDispatcher implements OnInit {

  protected readonly isMobile = isMobile();

  filters: BlockProductionWonSlotsFilters;
  totalWonSlots: number = 0;
  totalCanonical: number = 0;
  totalOrphaned: number = 0;
  totalFuture: number = 0;

  ngOnInit(): void {
    this.listenToFilters();
    this.listenToActiveEpoch();
  }

  private listenToFilters(): void {
    this.select(BlockProductionWonSlotsSelectors.filters, filters => {
      this.filters = filters;
      this.detect();
    });
  }

  private listenToActiveEpoch(): void {
    this.select(BlockProductionWonSlotsSelectors.slots, slots => {
      this.totalWonSlots = slots.length;
      this.totalCanonical = slots.filter(s => s.status === BlockProductionWonSlotsStatus.Canonical).length;
      this.totalOrphaned = slots.filter(s => s.status === BlockProductionWonSlotsStatus.Orphaned || s.status == BlockProductionWonSlotsStatus.Discarded).length;
      this.totalFuture = slots.filter(s => !s.status || s.status === BlockProductionWonSlotsStatus.Scheduled).length;
      this.detect();
    });
  }

  changeFilter(filter: keyof BlockProductionWonSlotsFilters, value: boolean): void {
    this.dispatch2(BlockProductionWonSlotsActions.changeFilters({ filters: { ...this.filters, [filter]: value } }));
  }

}
