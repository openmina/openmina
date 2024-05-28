import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import { BLOCK_PRODUCTION_OVERVIEW_KEY } from '@block-production/overview/block-production-overview.actions';
import { BLOCK_PRODUCTION_WON_SLOTS_KEY } from '@block-production/won-slots/block-production-won-slots.actions';
import { BlockProductionWonSlotsState } from '@block-production/won-slots/block-production-won-slots.state';
import { BLOCK_PRODUCTION_KEY } from '@block-production/block-production.actions';

const select = <T>(selector: (state: BlockProductionState) => T): MemoizedSelector<MinaState, T> => createSelector(
  createFeatureSelector<BlockProductionState>(BLOCK_PRODUCTION_KEY),
  selector,
);

const overview = select((state: BlockProductionState): BlockProductionOverviewState => state[BLOCK_PRODUCTION_OVERVIEW_KEY]);
const wonSlots = select((state: BlockProductionState): BlockProductionWonSlotsState => state[BLOCK_PRODUCTION_WON_SLOTS_KEY]);
export const BlockProductionSelectors = {
  overview,
  wonSlots,
};

export interface BlockProductionState {
  [BLOCK_PRODUCTION_OVERVIEW_KEY]: BlockProductionOverviewState;
  [BLOCK_PRODUCTION_WON_SLOTS_KEY]: BlockProductionWonSlotsState;
}
