import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import { BlockProductionOverviewActions } from '@block-production/overview/block-production-overview.actions';
import { createReducer, on } from '@ngrx/store';

const initialState: BlockProductionOverviewState = {
  epochs: [],
  activeEpoch: undefined,
  allTimeStats: undefined,
  activeEpochNumber: undefined,
  filters: {
    canonical: true,
    orphaned: true,
    missed: true,
    future: true,
  },
  scale: 'adaptive',
};

export const blockProductionOverviewReducer = createReducer(
  initialState,
  on(BlockProductionOverviewActions.getEpochsSuccess, (state, { epochs }) => ({
    ...state,
    epochs,
    activeEpoch: {
      ...state.activeEpoch,
      ...epochs.find(e => e.epochNumber === state.activeEpochNumber),
    },
  })),
  on(BlockProductionOverviewActions.getEpochDetails, (state, { epochNumber }) => ({
    ...state,
    activeEpochNumber: epochNumber,
  })),
  on(BlockProductionOverviewActions.getEpochDetailsSuccess, (state, { details }) => ({
    ...state,
    activeEpoch: {
      ...state.activeEpoch,
      details,
    },
    activeEpochNumber: state.activeEpochNumber || details.epochNumber,
  })),
  on(BlockProductionOverviewActions.getSlotsSuccess, (state, { slots }) => ({
    ...state,
    activeEpoch: {
      ...state.activeEpoch,
      slots,
    },
  })),
  on(BlockProductionOverviewActions.changeFilters, (state, { filters }) => ({
    ...state,
    filters,
  })),
  on(BlockProductionOverviewActions.getRewardsStatsSuccess, (state, { stats }) => ({
    ...state,
    allTimeStats: stats,
  })),
  on(BlockProductionOverviewActions.changeScale, (state, { scale }) => ({
    ...state,
    scale,
  })),
  on(BlockProductionOverviewActions.close, () => initialState),
);
