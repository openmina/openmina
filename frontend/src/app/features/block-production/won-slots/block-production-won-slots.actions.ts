import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import { createAction, props } from '@ngrx/store';
import { createType } from '@shared/constants/store-functions';
import { BLOCK_PRODUCTION_PREFIX } from '@block-production/block-production.actions';
import {
  BlockProductionWonSlotsSlot,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import {
  BlockProductionWonSlotsFilters,
} from '@shared/types/block-production/won-slots/block-production-won-slots-filters.type';
import { TableSort } from '@openmina/shared';

export const BLOCK_PRODUCTION_WON_SLOTS_KEY = 'wonSlots';

const type = <T extends string>(type: T) => createType(BLOCK_PRODUCTION_PREFIX, 'Won Slots', type);

const init = createAction(type('Init'));
const close = createAction(type('Close'));
const getActiveEpoch = createAction(type('Get Active Epoch'));
const getActiveEpochSuccess = createAction(type('Get Active Epoch Success'), props<{
  epoch: BlockProductionOverviewEpochDetails
}>());
const getSlots = createAction(type('Get Slots'));
const getSlotsSuccess = createAction(type('Get Slots Success'), props<{ slots: BlockProductionWonSlotsSlot[] }>());
const changeFilters = createAction(type('Change Filters'), props<{ filters: BlockProductionWonSlotsFilters }>());
const setActiveSlot = createAction(type('Set Active Slot'), props<{ slot: BlockProductionWonSlotsSlot }>());
const sort = createAction(type('Sort'), props<{ sort: TableSort<BlockProductionWonSlotsSlot> }>());

export const BlockProductionWonSlotsActions = {
  init,
  close,
  getActiveEpoch,
  getActiveEpochSuccess,
  getSlots,
  getSlotsSuccess,
  changeFilters,
  setActiveSlot,
  sort,
};
