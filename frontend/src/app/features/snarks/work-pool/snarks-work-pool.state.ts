import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { TableSort } from '@openmina/shared';
import { WorkPool } from '@shared/types/snarks/work-pool/work-pool.type';
import { WorkPoolSpecs } from '@shared/types/snarks/work-pool/work-pool-specs.type';
import { WorkPoolDetail } from '@shared/types/snarks/work-pool/work-pool-detail.type';
import { selectSnarksWorkPoolState } from '@snarks/snarks.state';

export interface SnarksWorkPoolState {
  workPools: WorkPool[];
  filteredWorkPools: WorkPool[];
  activeWorkPool: WorkPool;
  openSidePanel: boolean;
  sort: TableSort<WorkPool>;
  filters: string[];
  activeWorkPoolSpecs: WorkPoolSpecs;
  activeWorkPoolDetail: WorkPoolDetail;
}

const select = <T>(selector: (state: SnarksWorkPoolState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectSnarksWorkPoolState,
  selector,
);

export const selectSnarksWorkPools = select((state: SnarksWorkPoolState): WorkPool[] => state.filteredWorkPools);
export const selectSnarksWorkPoolActiveWorkPool = select((state: SnarksWorkPoolState): WorkPool => state.activeWorkPool);
export const selectSnarksWorkPoolOpenSidePanel = select((state: SnarksWorkPoolState): boolean => state.openSidePanel);
export const selectSnarksWorkPoolSort = select((state: SnarksWorkPoolState): TableSort<WorkPool> => state.sort);
export const selectSnarksWorkPoolFilters = select((state: SnarksWorkPoolState): string[] => state.filters);
export const selectSnarksWorkPoolActiveWorkPoolSpecs = select((state: SnarksWorkPoolState): WorkPoolSpecs => state.activeWorkPoolSpecs);
export const selectSnarksWorkPoolActiveWorkPoolDetail = select((state: SnarksWorkPoolState): WorkPoolDetail => state.activeWorkPoolDetail);
