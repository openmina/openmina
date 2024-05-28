import { ActionReducer, combineReducers } from '@ngrx/store';
import { BlockProductionState } from '@block-production/block-production.state';

import { blockProductionOverviewReducer } from '@block-production/overview/block-production-overview.reducer';
import { BLOCK_PRODUCTION_OVERVIEW_KEY } from '@block-production/overview/block-production-overview.actions';
import { blockProductionWonSlotsReducer } from '@block-production/won-slots/block-production-won-slots.reducer';
import { BLOCK_PRODUCTION_WON_SLOTS_KEY } from '@block-production/won-slots/block-production-won-slots.actions';

export const blockProductionReducer: ActionReducer<BlockProductionState> = combineReducers<BlockProductionState>({
  [BLOCK_PRODUCTION_OVERVIEW_KEY]: blockProductionOverviewReducer,
  [BLOCK_PRODUCTION_WON_SLOTS_KEY]: blockProductionWonSlotsReducer,
});
