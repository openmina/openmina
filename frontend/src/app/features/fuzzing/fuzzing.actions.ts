import { FuzzingFile } from '@shared/types/fuzzing/fuzzing-file.type';
import { FuzzingFileDetails } from '@shared/types/fuzzing/fuzzing-file-details.type';
import { FuzzingDirectory } from '@shared/types/fuzzing/fuzzing-directory.type';
import { FeatureAction, TableSort } from '@openmina/shared';

enum FuzzingActionTypes {
  FUZZING_INIT = 'FUZZING_INIT',
  FUZZING_CLOSE = 'FUZZING_CLOSE',
  FUZZING_GET_DIRECTORIES = 'FUZZING_GET_DIRECTORIES',
  FUZZING_GET_DIRECTORIES_SUCCESS = 'FUZZING_GET_DIRECTORIES_SUCCESS',
  FUZZING_SET_ACTIVE_DIRECTORY = 'FUZZING_SET_ACTIVE_DIRECTORY',
  FUZZING_GET_FILES = 'FUZZING_GET_FILES',
  FUZZING_GET_FILES_SUCCESS = 'FUZZING_GET_FILES_SUCCESS',
  FUZZING_GET_FILE_DETAILS = 'FUZZING_GET_FILE_DETAILS',
  FUZZING_GET_FILE_DETAILS_SUCCESS = 'FUZZING_GET_FILE_DETAILS_SUCCESS',
  FUZZING_SORT = 'FUZZING_SORT',
  FUZZING_FILTER = 'FUZZING_FILTER',
}

export const FUZZING_INIT = FuzzingActionTypes.FUZZING_INIT;
export const FUZZING_CLOSE = FuzzingActionTypes.FUZZING_CLOSE;
export const FUZZING_GET_DIRECTORIES = FuzzingActionTypes.FUZZING_GET_DIRECTORIES;
export const FUZZING_GET_DIRECTORIES_SUCCESS = FuzzingActionTypes.FUZZING_GET_DIRECTORIES_SUCCESS;
export const FUZZING_SET_ACTIVE_DIRECTORY = FuzzingActionTypes.FUZZING_SET_ACTIVE_DIRECTORY;
export const FUZZING_GET_FILES = FuzzingActionTypes.FUZZING_GET_FILES;
export const FUZZING_GET_FILES_SUCCESS = FuzzingActionTypes.FUZZING_GET_FILES_SUCCESS;
export const FUZZING_GET_FILE_DETAILS = FuzzingActionTypes.FUZZING_GET_FILE_DETAILS;
export const FUZZING_GET_FILE_DETAILS_SUCCESS = FuzzingActionTypes.FUZZING_GET_FILE_DETAILS_SUCCESS;
export const FUZZING_SORT = FuzzingActionTypes.FUZZING_SORT;
export const FUZZING_FILTER = FuzzingActionTypes.FUZZING_FILTER;

export interface FuzzingAction extends FeatureAction<FuzzingActionTypes> {
  readonly type: FuzzingActionTypes;
}

export class FuzzingInit implements FuzzingAction {
  readonly type = FUZZING_INIT;
}

export class FuzzingClose implements FuzzingAction {
  readonly type = FUZZING_CLOSE;
}

export class FuzzingGetDirectories implements FuzzingAction {
  readonly type = FUZZING_GET_DIRECTORIES;
}

export class FuzzingGetDirectoriesSuccess implements FuzzingAction {
  readonly type = FUZZING_GET_DIRECTORIES_SUCCESS;

  constructor(public payload: FuzzingDirectory[]) { }
}

export class FuzzingSetActiveDirectory implements FuzzingAction {
  readonly type = FUZZING_SET_ACTIVE_DIRECTORY;

  constructor(public payload: FuzzingDirectory) { }
}

export class FuzzingGetFiles implements FuzzingAction {
  readonly type = FUZZING_GET_FILES;
}

export class FuzzingGetFilesSuccess implements FuzzingAction {
  readonly type = FUZZING_GET_FILES_SUCCESS;

  constructor(public payload: FuzzingFile[]) { }
}

export class FuzzingGetFileDetails implements FuzzingAction {
  readonly type = FUZZING_GET_FILE_DETAILS;

  constructor(public payload: FuzzingFile) { }
}

export class FuzzingGetFileDetailsSuccess implements FuzzingAction {
  readonly type = FUZZING_GET_FILE_DETAILS_SUCCESS;

  constructor(public payload: FuzzingFileDetails) { }
}

export class FuzzingSort implements FuzzingAction {
  readonly type = FUZZING_SORT;

  constructor(public payload: TableSort<FuzzingFile>) { }
}

export class FuzzingFilterFiles implements FuzzingAction {
  readonly type = FUZZING_FILTER;

  constructor(public payload: string) { }
}

export type FuzzingActions =
  | FuzzingInit
  | FuzzingClose
  | FuzzingGetDirectories
  | FuzzingGetDirectoriesSuccess
  | FuzzingSetActiveDirectory
  | FuzzingGetFiles
  | FuzzingGetFilesSuccess
  | FuzzingGetFileDetails
  | FuzzingGetFileDetailsSuccess
  | FuzzingSort
  | FuzzingFilterFiles
  ;
