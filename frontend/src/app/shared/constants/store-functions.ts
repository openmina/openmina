import { catchError, Observable, of, OperatorFunction, repeat } from 'rxjs';
import { ADD_ERROR, ErrorAdd } from '@error-preview/error-preview.actions';
import { HttpErrorResponse } from '@angular/common/http';
import { FeatureAction, toReadableDate } from '@openmina/shared';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';

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
