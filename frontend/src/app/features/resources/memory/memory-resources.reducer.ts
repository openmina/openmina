import { MemoryResourcesState } from '@resources/memory/memory-resources.state';
import {
  MEMORY_RESOURCES_CLOSE,
  MEMORY_RESOURCES_GET_SUCCESS,
  MEMORY_RESOURCES_SET_ACTIVE_RESOURCE,
  MEMORY_RESOURCES_SET_GRANULARITY,
  MEMORY_RESOURCES_SET_TREEMAP_VIEW,
  MemoryResourcesActions,
} from '@resources/memory/memory-resources.actions';
import { TreemapView } from '@shared/types/resources/memory/treemap-view.type';

const initialState: MemoryResourcesState = {
  resource: undefined,
  activeResource: undefined,
  breadcrumbs: [],
  granularity: Number(localStorage.getItem('memory-granularity')) || 512,
  treemapView: localStorage.getItem('memory-view') as TreemapView || TreemapView.BINARY,
};

export function memoryResourcesReducer(state: MemoryResourcesState = initialState, action: MemoryResourcesActions): MemoryResourcesState {
  switch (action.type) {
    case MEMORY_RESOURCES_GET_SUCCESS: {
      return {
        ...state,
        resource: action.payload,
        breadcrumbs: action.payload ? [action.payload] : [],
        activeResource: action.payload,
      };
    }

    case MEMORY_RESOURCES_SET_ACTIVE_RESOURCE: {
      if (state.activeResource === action.payload) {
        return state;
      }
      let breadcrumbs = state.breadcrumbs;
      if (action.payload.name.executableName === 'root') {
        breadcrumbs = [state.resource];
      } else {
        const indexOfSameResource = breadcrumbs.findIndex(b => b.id === action.payload.id);
        if (indexOfSameResource !== -1) {
          breadcrumbs = breadcrumbs.slice(0, indexOfSameResource + 1);
        } else {
          breadcrumbs = [...breadcrumbs, action.payload];
        }
      }
      return {
        ...state,
        activeResource: action.payload,
        breadcrumbs,
      };
    }

    case MEMORY_RESOURCES_SET_GRANULARITY: {
      localStorage.setItem('memory-granularity', String(action.payload));
      return {
        ...state,
        granularity: action.payload,
        resource: undefined,
        activeResource: undefined,
        breadcrumbs: [],
      };
    }

    case MEMORY_RESOURCES_SET_TREEMAP_VIEW: {
      localStorage.setItem('memory-view', String(action.payload));
      return {
        ...state,
        treemapView: action.payload,
      };
    }

    case MEMORY_RESOURCES_CLOSE:
      return initialState;

    default:
      return state;
  }
}
