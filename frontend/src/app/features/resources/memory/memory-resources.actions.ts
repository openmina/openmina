import { FeatureAction } from '@openmina/shared';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import { TreemapView } from '@shared/types/resources/memory/treemap-view.type';

enum MemoryResourcesActionTypes {
  MEMORY_RESOURCES_GET = 'MEMORY_RESOURCES_GET',
  MEMORY_RESOURCES_GET_SUCCESS = 'MEMORY_RESOURCES_GET_SUCCESS',
  MEMORY_RESOURCES_SET_ACTIVE_RESOURCE = 'MEMORY_RESOURCES_SET_ACTIVE_RESOURCE',
  MEMORY_RESOURCES_SET_GRANULARITY = 'MEMORY_RESOURCES_SET_GRANULARITY',
  MEMORY_RESOURCES_SET_TREEMAP_VIEW = 'MEMORY_RESOURCES_SET_TREEMAP_VIEW',
  MEMORY_RESOURCES_CLOSE = 'MEMORY_RESOURCES_CLOSE',
}

export const MEMORY_RESOURCES_GET = MemoryResourcesActionTypes.MEMORY_RESOURCES_GET;
export const MEMORY_RESOURCES_GET_SUCCESS = MemoryResourcesActionTypes.MEMORY_RESOURCES_GET_SUCCESS;
export const MEMORY_RESOURCES_SET_ACTIVE_RESOURCE = MemoryResourcesActionTypes.MEMORY_RESOURCES_SET_ACTIVE_RESOURCE;
export const MEMORY_RESOURCES_SET_GRANULARITY = MemoryResourcesActionTypes.MEMORY_RESOURCES_SET_GRANULARITY;
export const MEMORY_RESOURCES_SET_TREEMAP_VIEW = MemoryResourcesActionTypes.MEMORY_RESOURCES_SET_TREEMAP_VIEW;
export const MEMORY_RESOURCES_CLOSE = MemoryResourcesActionTypes.MEMORY_RESOURCES_CLOSE;

export interface MemoryResourcesAction extends FeatureAction<MemoryResourcesActionTypes> {
  readonly type: MemoryResourcesActionTypes;
}

export class MemoryResourcesGet implements MemoryResourcesAction {
  readonly type = MEMORY_RESOURCES_GET;
}

export class MemoryResourcesGetSuccess implements MemoryResourcesAction {
  readonly type = MEMORY_RESOURCES_GET_SUCCESS;

  constructor(public payload: MemoryResource) {}
}

export class MemoryResourcesSetActiveResource implements MemoryResourcesAction {
  readonly type = MEMORY_RESOURCES_SET_ACTIVE_RESOURCE;

  constructor(public payload: MemoryResource) {}
}

export class MemoryResourcesSetGranularity implements MemoryResourcesAction {
  readonly type = MEMORY_RESOURCES_SET_GRANULARITY;

  constructor(public payload: number) {}
}

export class MemoryResourcesSetTreemapView implements MemoryResourcesAction {
  readonly type = MEMORY_RESOURCES_SET_TREEMAP_VIEW;

  constructor(public payload: TreemapView) {}
}

export class MemoryResourcesClose implements MemoryResourcesAction {
  readonly type = MEMORY_RESOURCES_CLOSE;
}


export type MemoryResourcesActions =
  | MemoryResourcesGet
  | MemoryResourcesGetSuccess
  | MemoryResourcesSetActiveResource
  | MemoryResourcesSetGranularity
  | MemoryResourcesSetTreemapView
  | MemoryResourcesClose
  ;
