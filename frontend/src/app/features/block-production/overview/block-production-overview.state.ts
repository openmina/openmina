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

export interface BlockProductionOverviewState {
  epochs: BlockProductionOverviewEpoch[];
  activeEpoch: BlockProductionOverviewEpoch | undefined;
  activeEpochNumber: number | undefined;
  allTimeStats: BlockProductionOverviewAllStats;
  filters: BlockProductionOverviewFilters;
  scale: 'linear' | 'adaptive';
}


const select = <T>(selector: (state: BlockProductionOverviewState) => T): MemoizedSelector<MinaState, T> => createSelector(
  BlockProductionSelectors.overview,
  selector,
);

const epochs = select((state: BlockProductionOverviewState): BlockProductionOverviewEpoch[] => state.epochs);
const activeEpoch = select((state: BlockProductionOverviewState): BlockProductionOverviewEpoch => state.activeEpoch);
const allTimeStats = select((state: BlockProductionOverviewState): BlockProductionOverviewAllStats => state.allTimeStats);
const filters = select((state: BlockProductionOverviewState): BlockProductionOverviewFilters => state.filters);
const scale = select((state: BlockProductionOverviewState): 'linear' | 'adaptive' => state.scale);

export const BlockProductionOverviewSelectors = {
  epochs,
  activeEpoch,
  allTimeStats,
  filters,
  scale,
};
