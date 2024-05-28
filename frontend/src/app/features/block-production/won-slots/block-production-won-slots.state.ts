import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { BlockProductionSelectors } from '@block-production/block-production.state';
import {
  BlockProductionOverviewFilters,
} from '@shared/types/block-production/overview/block-production-overview-filters.type';
import {
  BlockProductionOverviewAllStats,
} from '@shared/types/block-production/overview/block-production-overview-all-stats.type';
import {
  BlockProductionWonSlotsFilters,
} from '@shared/types/block-production/won-slots/block-production-won-slots-filters.type';
import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import {
  BlockProductionWonSlotsSlot,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { TableSort } from '@openmina/shared';

export interface BlockProductionWonSlotsState {
  epoch: BlockProductionOverviewEpochDetails;
  slots: BlockProductionWonSlotsSlot[];
  filteredSlots: BlockProductionWonSlotsSlot[];
  activeSlot: BlockProductionWonSlotsSlot;
  filters: BlockProductionWonSlotsFilters;
  sort: TableSort<BlockProductionWonSlotsSlot>;
}


const select = <T>(selector: (state: BlockProductionWonSlotsState) => T): MemoizedSelector<MinaState, T> => createSelector(
  BlockProductionSelectors.wonSlots,
  selector,
);

const epoch = select(state => state.epoch);
const activeEpoch = select(state => state.epoch);
const slots = select(state => state.slots);
const filteredSlots = select(state => state.filteredSlots);
const activeSlot = select(state => state.activeSlot);
const filters = select(state => state.filters);
const sort = select(state => state.sort);

export const BlockProductionWonSlotsSelectors = {
  epoch,
  activeEpoch,
  slots,
  filteredSlots,
  activeSlot,
  filters,
  sort,
};
