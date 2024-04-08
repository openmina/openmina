import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { selectBlockProductionOverviewState } from '@block-production/block-production.state';
import {
  BlockProductionOverviewFilters,
} from '@shared/types/block-production/overview/block-production-overview-filters.type';
import {
  BlockProductionOverviewAllStats,
} from '@shared/types/block-production/overview/block-production-overview-all-stats.type';

export interface BlockProductionOverviewState {
  epochs: BlockProductionOverviewEpoch[];
  activeEpoch: BlockProductionOverviewEpoch | undefined;
  activeEpochNumber: number | undefined;
  allTimeStats: BlockProductionOverviewAllStats;
  filters: BlockProductionOverviewFilters;
  scale: 'linear' | 'adaptive';
}

const select = <T>(selector: (state: BlockProductionOverviewState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectBlockProductionOverviewState,
  selector,
);

export const selectBlockProductionOverviewEpochs = select((state: BlockProductionOverviewState): BlockProductionOverviewEpoch[] => state.epochs);
export const selectBlockProductionOverviewActiveEpoch = select((state: BlockProductionOverviewState): BlockProductionOverviewEpoch => state.activeEpoch);
export const selectBlockProductionOverviewAllTimeStats = select((state: BlockProductionOverviewState): BlockProductionOverviewAllStats => state.allTimeStats);
export const selectBlockProductionOverviewFilters = select((state: BlockProductionOverviewState): BlockProductionOverviewFilters => state.filters);
export const selectBlockProductionOverviewScale = select((state: BlockProductionOverviewState): 'linear' | 'adaptive' => state.scale);
