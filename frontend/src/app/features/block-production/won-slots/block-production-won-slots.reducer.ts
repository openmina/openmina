import { createReducer, on } from '@ngrx/store';
import { BlockProductionWonSlotsState } from '@block-production/won-slots/block-production-won-slots.state';
import { sort, SortDirection, TableSort } from '@openmina/shared';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';
import {
  BlockProductionWonSlotsSlot,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';

const initialState: BlockProductionWonSlotsState = {
  epoch: undefined,
  slots: [],
  filteredSlots: [],
  activeSlot: undefined,
  filters: {
    accepted: true,
    rejected: true,
    upcoming: true,
    missed: true,
  },
  sort: {
    sortBy: 'slotTime',
    sortDirection: SortDirection.ASC,
  },
};

export const blockProductionWonSlotsReducer = createReducer(
  initialState,
  on(BlockProductionWonSlotsActions.getActiveEpochSuccess, (state, { epoch }) => ({
    ...state,
    epoch,
  })),
  on(BlockProductionWonSlotsActions.getSlotsSuccess, (state, { slots }) => ({
    ...state,
    slots,
    filteredSlots: filterSlots(sortSlots(slots, state.sort), state.filters),
    activeSlot: slots.find(s => s.active) || state.activeSlot,
  })),
  on(BlockProductionWonSlotsActions.setActiveSlot, (state, { slot }) => ({
    ...state,
    activeSlot: slot,
  })),
  on(BlockProductionWonSlotsActions.sort, (state, { sort }) => ({
    ...state,
    sort,
    filteredSlots: filterSlots(sortSlots(state.slots, sort), state.filters),
  })),
  on(BlockProductionWonSlotsActions.changeFilters, (state, { filters }) => ({
    ...state,
    filters,
    filteredSlots: filterSlots(sortSlots(state.slots, state.sort), filters),
  })),
  on(BlockProductionWonSlotsActions.close, () => initialState),
);

function sortSlots(node: BlockProductionWonSlotsSlot[], tableSort: TableSort<BlockProductionWonSlotsSlot>): BlockProductionWonSlotsSlot[] {
  return sort<BlockProductionWonSlotsSlot>(node, tableSort, ['message']);
}

function filterSlots(node: BlockProductionWonSlotsSlot[], filters: BlockProductionWonSlotsState['filters']): BlockProductionWonSlotsSlot[] {
  return node.filter(slot => {
    // if (
    //   (filters.accepted && slot.canonical)
    //   || (filters.rejected && slot.orphaned)
    //   || (filters.missed && slot.missed)
    //   || slot.active
    // ) {
    //   return true;
    // }
    // return filters.upcoming && slot.futureRights;
    return true;
  });
}
