import { FeatureAction } from '@openmina/shared';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import {
  BlockProductionOverviewFilters,
} from '@shared/types/block-production/overview/block-production-overview-filters.type';
import { BlockProductionSlot } from '@shared/types/block-production/overview/block-production-overview-slot.type';
import {
  BlockProductionOverviewAllStats,
} from '@shared/types/block-production/overview/block-production-overview-all-stats.type';

enum BlockProductionOverviewActionTypes {
  BLOCK_PRODUCTION_OVERVIEW_INIT = 'BLOCK_PRODUCTION_OVERVIEW_INIT',
  BLOCK_PRODUCTION_OVERVIEW_CLOSE = 'BLOCK_PRODUCTION_OVERVIEW_CLOSE',
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS = 'BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS',
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS = 'BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS',
  BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS = 'BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS',
  BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS = 'BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS',
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS = 'BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS',
  BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS = 'BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS',
  BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS = 'BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS',
  BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS = 'BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS',
  BLOCK_PRODUCTION_OVERVIEW_CHANGE_FILTERS = 'BLOCK_PRODUCTION_OVERVIEW_CHANGE_FILTERS',
  BLOCK_PRODUCTION_OVERVIEW_CHANGE_SCALE = 'BLOCK_PRODUCTION_OVERVIEW_CHANGE_SCALE',
}

export const BLOCK_PRODUCTION_OVERVIEW_INIT = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_INIT;
export const BLOCK_PRODUCTION_OVERVIEW_CLOSE = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_CLOSE;
export const BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS;
export const BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS;
export const BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS;
export const BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS;
export const BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS;
export const BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS;
export const BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS;
export const BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS;
export const BLOCK_PRODUCTION_OVERVIEW_CHANGE_FILTERS = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_CHANGE_FILTERS;
export const BLOCK_PRODUCTION_OVERVIEW_CHANGE_SCALE = BlockProductionOverviewActionTypes.BLOCK_PRODUCTION_OVERVIEW_CHANGE_SCALE;

export interface BlockProductionOverviewAction extends FeatureAction<BlockProductionOverviewActionTypes> {
  readonly type: BlockProductionOverviewActionTypes;
}

export class BlockProductionOverviewInit implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_INIT;
}

export class BlockProductionOverviewClose implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_CLOSE;
}

export class BlockProductionOverviewGetEpochs implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS;

  constructor(public payload?: number) {}
}

export class BlockProductionOverviewGetEpochsSuccess implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_EPOCHS_SUCCESS;

  constructor(public payload: BlockProductionOverviewEpoch[]) {}
}

export class BlockProductionOverviewGetSlots implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS;

  constructor(public payload: number) {}
}

export class BlockProductionOverviewGetSlotsSuccess implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_SLOTS_SUCCESS;

  constructor(public payload: BlockProductionSlot[]) {}
}

export class BlockProductionOverviewGetEpochDetails implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS;

  constructor(public payload: number) {}
}

export class BlockProductionOverviewGetEpochDetailsSuccess implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_EPOCH_DETAILS_SUCCESS;

  constructor(public payload: BlockProductionOverviewEpochDetails) {}
}

export class BlockProductionOverviewGetRewardsStats implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS;
}

export class BlockProductionOverviewGetRewardsStatsSuccess implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_GET_REWARDS_STATS_SUCCESS;

  constructor(public payload: BlockProductionOverviewAllStats) {}
}

export class BlockProductionOverviewChangeFilters implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_CHANGE_FILTERS;

  constructor(public payload: BlockProductionOverviewFilters) {}
}

export class BlockProductionOverviewChangeScale implements BlockProductionOverviewAction {
  readonly type = BLOCK_PRODUCTION_OVERVIEW_CHANGE_SCALE;

  constructor(public payload: 'linear' | 'adaptive') {}
}

export type BlockProductionOverviewActions =
  | BlockProductionOverviewInit
  | BlockProductionOverviewClose
  | BlockProductionOverviewGetEpochs
  | BlockProductionOverviewGetEpochsSuccess
  | BlockProductionOverviewGetSlots
  | BlockProductionOverviewGetSlotsSuccess
  | BlockProductionOverviewGetEpochDetails
  | BlockProductionOverviewGetEpochDetailsSuccess
  | BlockProductionOverviewGetRewardsStats
  | BlockProductionOverviewGetRewardsStatsSuccess
  | BlockProductionOverviewChangeFilters
  | BlockProductionOverviewChangeScale
  ;
