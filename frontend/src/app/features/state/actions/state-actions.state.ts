import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { StateActionGroup } from '@shared/types/state/actions/state-action-group.type';
import { TableSort } from '@openmina/shared';
import { StateActionsStats } from '@shared/types/state/actions/state-actions-stats.type';
import { selectStateActionsState } from '@state/state.state';

export interface StateActionsState {
  groups: StateActionGroup[];
  filteredGroups: StateActionGroup[];
  openSidePanel: boolean;
  earliestSlot: number;
  activeSlot: number;
  currentSort: TableSort<StateActionGroup>;
  activeSearch: string;
  stats: StateActionsStats
}

const select = <T>(selector: (state: StateActionsState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectStateActionsState,
  selector,
);

export const selectStateActionsGroups = select((state: StateActionsState): StateActionGroup[] => state.filteredGroups);
export const selectStateActionsOpenSidePanel = select((state: StateActionsState): boolean => state.openSidePanel);
export const selectStateActionsActiveSlotAndStats = select((state: StateActionsState): [number, StateActionsStats] => [state.activeSlot, state.stats]);
export const selectStateActionsToolbarValues = select((state: StateActionsState): {
  earliestSlot: number;
  activeSlot: number;
  currentSort: TableSort<StateActionGroup>;
  activeSearch: string;
} => ({
  earliestSlot: state.earliestSlot,
  activeSlot: state.activeSlot,
  currentSort: state.currentSort,
  activeSearch: state.activeSearch,
}));
export const selectStateActionsSort = select((state: StateActionsState): TableSort<StateActionGroup> => state.currentSort);

