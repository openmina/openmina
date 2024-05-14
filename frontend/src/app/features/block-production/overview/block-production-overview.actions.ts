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
import { createAction, props } from '@ngrx/store';
import { createType } from '@shared/constants/store-functions';
import { BLOCK_PRODUCTION_PREFIX } from '@block-production/block-production.actions';

export const BLOCK_PRODUCTION_OVERVIEW_KEY = 'overview';

const type = <T extends string>(type: T) => createType(BLOCK_PRODUCTION_PREFIX, 'Overview', type);

const init = createAction(type('Init'));
const close = createAction(type('Close'));
const getEpochs = createAction(type('Get Epochs'), props<{ epochNumber: number }>());
const getEpochsSuccess = createAction(type('Get Epochs Success'), props<{ epochs: BlockProductionOverviewEpoch[] }>());
const getSlots = createAction(type('Get Slots'), props<{ epochNumber: number }>());
const getSlotsSuccess = createAction(type('Get Slots Success'), props<{ slots: BlockProductionSlot[] }>());
const getEpochDetails = createAction(type('Get Epoch Details'), props<{ epochNumber: number }>());
const getEpochDetailsSuccess = createAction(type('Get Epoch Details Success'), props<{
  details: BlockProductionOverviewEpochDetails
}>());
const getRewardsStats = createAction(type('Get Rewards Stats'));
const getRewardsStatsSuccess = createAction(type('Get Rewards Stats Success'), props<{
  stats: BlockProductionOverviewAllStats
}>());
const changeFilters = createAction(type('Change Filters'), props<{ filters: BlockProductionOverviewFilters }>());
const changeScale = createAction(type('Change Scale'), props<{ scale: 'linear' | 'adaptive' }>());

export const BlockProductionOverviewActions = {
  init,
  close,
  getEpochs,
  getEpochsSuccess,
  getSlots,
  getSlotsSuccess,
  getEpochDetails,
  getEpochDetailsSuccess,
  getRewardsStats,
  getRewardsStatsSuccess,
  changeFilters,
  changeScale,
};
