import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import {
  BLOCK_PRODUCTION_OVERVIEW_CHANGE_FILTERS, BLOCK_PRODUCTION_OVERVIEW_CHANGE_SCALE,
  BLOCK_PRODUCTION_OVERVIEW_CLOSE,
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS,
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS,
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS,
  BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS,
  BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS,
  BlockProductionOverviewActions,
} from '@block-production/overview/block-production-overview.actions';

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

export function blockProductionOverviewReducer(state: BlockProductionOverviewState = initialState, action: BlockProductionOverviewActions): BlockProductionOverviewState {

  switch (action.type) {

    case BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS:
      return {
        ...state,
        epochs: action.payload,
        activeEpoch: {
          ...state.activeEpoch,
          ...action.payload.find(e => e.epochNumber === state.activeEpochNumber),
        },
      };

    case BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS:
      return {
        ...state,
        activeEpochNumber: action.payload,
      };

    case BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS:
      return {
        ...state,
        activeEpoch: {
          ...state.activeEpoch,
          details: action.payload,
        },
        activeEpochNumber: state.activeEpochNumber || action.payload.epochNumber,
      };

    case BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS:
      return {
        ...state,
        activeEpoch: {
          ...state.activeEpoch,
          slots: action.payload,
        },
      };

    case BLOCK_PRODUCTION_OVERVIEW_CHANGE_FILTERS:
      return {
        ...state,
        filters: action.payload,
      };

    case BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS:
      return {
        ...state,
        allTimeStats: action.payload,
      };

    case BLOCK_PRODUCTION_OVERVIEW_CHANGE_SCALE:
      return {
        ...state,
        scale: action.payload,
      };

    case BLOCK_PRODUCTION_OVERVIEW_CLOSE:
      return initialState;

    default:
      return state;
  }
}
