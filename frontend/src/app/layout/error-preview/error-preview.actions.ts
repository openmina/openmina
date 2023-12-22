import { FeatureAction } from '@openmina/shared';
import { MinaError } from '@shared/types/error-preview/mina-error.type';

enum ErrorPreviewActionTypes {
  ADD_ERROR = 'ADD_ERROR',
  MARK_ERRORS_AS_SEEN = 'MARK_ERRORS_AS_SEEN',
}

export const ADD_ERROR = ErrorPreviewActionTypes.ADD_ERROR;
export const MARK_ERRORS_AS_SEEN = ErrorPreviewActionTypes.MARK_ERRORS_AS_SEEN;

export interface ErrorPreviewAction extends FeatureAction<ErrorPreviewActionTypes> {
  readonly type: ErrorPreviewActionTypes;
}

export class ErrorAdd implements ErrorPreviewAction {
  readonly type = ADD_ERROR;

  constructor(public payload?: MinaError) {}
}

export class MarkErrorsAsSeen implements ErrorPreviewAction {
  readonly type = MARK_ERRORS_AS_SEEN;
}

export type ErrorPreviewActions =
  | ErrorAdd
  | MarkErrorsAsSeen
  ;
