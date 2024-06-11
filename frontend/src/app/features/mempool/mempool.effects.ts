import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { EMPTY, map, switchMap } from 'rxjs';
import { catchErrorAndRepeat2 } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { BaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import { MempoolActions } from '@app/features/mempool/mempool.actions';
import { MempoolService } from '@app/features/mempool/mempool.service';

@Injectable({
  providedIn: 'root',
})
export class MempoolEffects extends BaseEffect {

  readonly init$: Effect;
  readonly getTxs$: Effect;

  constructor(private actions$: Actions,
              private mempoolService: MempoolService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(MempoolActions.init),
      map(() => MempoolActions.getTxs()),
    ));

    this.getTxs$ = createEffect(() => this.actions$.pipe(
      ofType(MempoolActions.getTxs, MempoolActions.close),
      this.latestActionState(),
      switchMap(({ action }) =>
        action.type === MempoolActions.close.type
          ? EMPTY
          : this.mempoolService.getTransactionPool(),
      ),
      map(data => MempoolActions.getTxsSuccess(data)),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, MempoolActions.getTxsSuccess({ txs: [] })),
    ));
  }
}
