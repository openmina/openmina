import { ActionReducer, combineReducers } from '@ngrx/store';
import { MemoryResourcesAction, MemoryResourcesActions } from '@resources/memory/memory-resources.actions';
import { memoryResourcesReducer } from '@resources/memory/memory-resources.reducer';
import { ResourcesState } from '@resources/resources.state';

export type ResourcesActions =
  & MemoryResourcesActions
  ;
export type ResourcesAction =
  & MemoryResourcesAction
  ;

export const resourcesReducer: ActionReducer<ResourcesState, ResourcesActions> = combineReducers<ResourcesState, ResourcesActions>({
  memory: memoryResourcesReducer,
});
