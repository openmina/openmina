import { ErrorPreviewState } from '@error-preview/error-preview.state';
import { ADD_ERROR, ErrorPreviewActions, MARK_ERRORS_AS_SEEN } from '@app/layout/error-preview/error-preview.actions';

const initialState: ErrorPreviewState = {
  errors: [],
};

export function reducer(state: ErrorPreviewState = initialState, action: ErrorPreviewActions): ErrorPreviewState {
  switch (action.type) {

    case ADD_ERROR: {
      return {
        ...state,
        errors: [action.payload, ...state.errors],
      };
    }

    case MARK_ERRORS_AS_SEEN: {
      return {
        ...state,
        errors: state.errors.map(e => ({ ...e, seen: true })),
      };
    }

    default:
      return state;
  }
}
