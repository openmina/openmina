import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import { selectMemoryResourcesState } from '@resources/resources.state';
import { TreemapView } from '@shared/types/resources/memory/treemap-view.type';

export interface MemoryResourcesState {
  resource: MemoryResource;
  activeResource: MemoryResource;
  breadcrumbs: MemoryResource[];
  granularity: number;
  treemapView: TreemapView;
}

const select = <T>(selector: (state: MemoryResourcesState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectMemoryResourcesState,
  selector,
);

export const selectMemoryResources = select((state: MemoryResourcesState): MemoryResource => state.resource);
export const selectMemoryResourcesActiveResource = select((state: MemoryResourcesState): MemoryResource => state.activeResource);
export const selectMemoryResourcesBreadcrumbs = select((state: MemoryResourcesState): MemoryResource[] => state.breadcrumbs);
export const selectMemoryResourcesGranularity = select((state: MemoryResourcesState): number => state.granularity);
export const selectMemoryResourcesTreemapView = select((state: MemoryResourcesState): TreemapView => state.treemapView);
