import { catchError, map, Observable, of, OperatorFunction, repeat } from 'rxjs';
import { ADD_ERROR, ErrorAdd } from '@error-preview/error-preview.actions';
import { HttpErrorResponse } from '@angular/common/http';
import { FeatureAction, toReadableDate } from '@openmina/shared';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Selector, Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { concatLatestFrom } from '@ngrx/effects';
import { TypedAction } from '@ngrx/store/src/models';

export const catchErrorAndRepeat = <T>(errType: MinaErrorType, type: string, payload?: any): OperatorFunction<T, ErrorAdd | T | FeatureAction<any>> => {
  return (source: Observable<T>) =>
    source.pipe(
      catchError((error: Error) => [
        addError(error, errType),
        { type, payload },
      ]),
      repeat(),
    );
};

export const addError = (error: HttpErrorResponse | Error, type: MinaErrorType): ErrorAdd => {
  console.error(error);
  return {
    type: ADD_ERROR,
    payload: {
      type,
      message: error.message,
      status: (error as any).status ? `${(error as any).status} ${(error as any).statusText}` : undefined,
      timestamp: toReadableDate(Number(Date.now()), 'HH:mm:ss'),
      seen: false,
    },
  } as ErrorAdd;
};

export const addErrorObservable = (error: HttpErrorResponse | Error | any, type: MinaErrorType): Observable<ErrorAdd> => of(addError(error, type));

export function createType<T extends string>(module: string, submodule: string, actionName: T): T {
  return `[${module} ${submodule}] ${actionName}` as T;
}

export const selectActionAndState = <A>(store: Store<MinaState>, selector: Selector<MinaState, any>): OperatorFunction<A, {
  action: A;
  state: MinaState
}> => (
  source$: Observable<A>,
): Observable<{ action: A; state: MinaState }> =>
  source$.pipe(
    concatLatestFrom(() => store.select(selector)),
    map(([action, state]: [A, MinaState]) => ({ action, state })),
  );

export const selectLatestStateSlice = <R extends object, A>(
  store: Store<MinaState>,
  selector: Selector<MinaState, any>,
  path: string,
): OperatorFunction<A, { action: A; state: R }> => (source$: Observable<A>): Observable<{ action: A; state: R }> =>
  source$.pipe(
    selectActionAndState(store, selector),
    map(({ action, state }: { action: A; state: MinaState }) => ({
      action,
      state: path.split('.').reduce((acc: any, key: string) => acc[key], state),
    })),
  );

export const catchErrorAndRepeat2 = <T>(errType: MinaErrorType, action?: {
  type: string;
  payload?: any
}): OperatorFunction<T, T | TypedAction<any>> => (source: Observable<T>) =>
  source.pipe(
    catchError((error: Error) => [
      addError(error, errType),
      action,
    ]),
    repeat(),
  );
