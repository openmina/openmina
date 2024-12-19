import { FuzzingState } from '@fuzzing/fuzzing.state';
import {
  FUZZING_CLOSE,
  FUZZING_FILTER,
  FUZZING_GET_DIRECTORIES_SUCCESS,
  FUZZING_GET_FILE_DETAILS,
  FUZZING_GET_FILE_DETAILS_SUCCESS,
  FUZZING_GET_FILES_SUCCESS,
  FUZZING_SET_ACTIVE_DIRECTORY,
  FUZZING_SORT,
  FuzzingActions,
} from '@fuzzing/fuzzing.actions';
import { FuzzingFile } from '@shared/types/fuzzing/fuzzing-file.type';
import { sort, SortDirection, TableSort } from '@openmina/shared';

const initialState: FuzzingState = {
  directories: [],
  activeDirectory: undefined,
  files: [],
  filteredFiles: [],
  activeFile: undefined,
  activeFileDetails: undefined,
  sort: {
    sortDirection: SortDirection.ASC,
    sortBy: 'path',
  },
  filterText: undefined,
};

export function fuzzingReducer(state: FuzzingState = initialState, action: FuzzingActions): FuzzingState {
  switch (action.type) {

    case FUZZING_GET_DIRECTORIES_SUCCESS: {
      if (JSON.stringify(action.payload) === JSON.stringify(state.directories)) {
        return state;
      }
      return {
        ...state,
        directories: action.payload,
      };
    }

    case FUZZING_SET_ACTIVE_DIRECTORY: {
      if (action.payload.fullName === state.activeDirectory?.fullName) {
        return state;
      }
      return {
        ...state,
        activeDirectory: action.payload,
        activeFile: undefined,
        activeFileDetails: undefined,
      };
    }

    case FUZZING_GET_FILES_SUCCESS: {
      const files = sortFiles(action.payload, state.sort);
      const filteredFiles = filterFiles(files, state.filterText);
      if (JSON.stringify(filteredFiles) === JSON.stringify(state.filteredFiles)) {
        return state;
      }
      return {
        ...state,
        files,
        filteredFiles,
      };
    }

    case FUZZING_GET_FILE_DETAILS: {
      return {
        ...state,
        activeFile: action.payload,
      };
    }

    case FUZZING_GET_FILE_DETAILS_SUCCESS: {
      if (JSON.stringify(action.payload) === JSON.stringify(state.activeFileDetails)) {
        return state;
      }
      return {
        ...state,
        activeFile: { ...state.activeFile },
        activeFileDetails: action.payload,
      };
    }

    case FUZZING_SORT: {
      return {
        ...state,
        filteredFiles: sortFiles(state.filteredFiles, action.payload),
        sort: action.payload,
      };
    }

    case FUZZING_FILTER: {
      const fuzzingFiles = filterFiles(state.files, action.payload);
      const filteredFiles = sortFiles(fuzzingFiles, state.sort);
      return {
        ...state,
        filterText: action.payload,
        filteredFiles,
      };
    }

    case FUZZING_CLOSE:
      return initialState;

    default:
      return state;
  }
}

function filterFiles(files: FuzzingFile[], filterText: string): FuzzingFile[] {
  return files.filter(file => !filterText || file.path.toLowerCase().includes(filterText));
}

function sortFiles(files: FuzzingFile[], tableSort: TableSort<FuzzingFile>): FuzzingFile[] {
  return sort<FuzzingFile>(files, tableSort, ['path']);
}
