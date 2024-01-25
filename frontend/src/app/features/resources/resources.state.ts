import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { SnarksWorkPoolState } from '@snarks/work-pool/snarks-work-pool.state';
import { ScanStateState } from '@snarks/scan-state/scan-state.state';
import { MemoryResourcesState } from '@resources/memory/memory-resources.state';

export interface ResourcesState {
  memory: MemoryResourcesState;
}

const select = <T>(selector: (state: ResourcesState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectResourcesState,
  selector,
);

export const selectResourcesState = createFeatureSelector<ResourcesState>('resources');
export const selectMemoryResourcesState = select((state: ResourcesState): MemoryResourcesState => state.memory);
