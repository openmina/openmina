import { createEffect } from '@ngrx/effects';
import { Store, Action } from '@ngrx/store';
import { map, Observable, ObservedValueOf, OperatorFunction } from 'rxjs';
import { Selector } from '@ngrx/store/src/models';
import { withLatestFrom } from 'rxjs/operators';
import { concatLatestFrom } from '@ngrx/operators';

export const createNonDispatchableEffect = (source: () => any) => createEffect(source, { dispatch: false });

// export const selectActionAndState = <S, A>(store: Store<S>, selector: Selector<S, any>): OperatorFunction<A, { action: A; state: S; }> =>
//   withLatestFrom(
//     store.select<S>(selector),
//     (action: A, state: ObservedValueOf<Store<S>>): { action: A, state: S } => ({ action, state }),
//   );


export const selectActionAndState = <S, A>(store: Store<S>, selector: Selector<S, any>): OperatorFunction<A, {
  action: A;
  state: S
}> => (
  source$: Observable<A>,
): Observable<{ action: A; state: S }> =>
  source$.pipe(
    concatLatestFrom(() => store.select(selector)),
    map(([action, state]: [A, S]) => ({ action, state })),
  );

// export const selectLatestStateSlice = <R extends object, S, A>(store: Store<S>, selector: Selector<S, any>, path: string): OperatorFunction<A, R> =>
//   withLatestFrom(
//     store.select<S>(selector),
//     (action: A, state: ObservedValueOf<Store<S>>): R => path.split('.').reduce((acc: R, key: string) => acc[key as keyof R], state as unknown as R),
//   );

export const selectLatestStateSlice = <R extends object, S, A>(
  store: Store<S>,
  selector: Selector<S, any>,
  path: string,
): OperatorFunction<A, { action: A; state: R }> => (source$: Observable<A>): Observable<{ action: A; state: R }> =>
  source$.pipe(
    selectActionAndState(store, selector),
    map(({ action, state }: { action: A; state: S }) => ({
      action,
      state: path.split('.').reduce((acc: any, key: string) => acc[key], state),
    })),
  );
