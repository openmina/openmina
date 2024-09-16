import { Injectable } from '@angular/core';
import { Effect, hasValue } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, forkJoin, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import {
  SNARKS_WORK_POOL_CLOSE,
  SNARKS_WORK_POOL_GET_WORK_POOL,
  SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL,
  SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS,
  SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS,
  SNARKS_WORK_POOL_SET_ACTIVE_WORK_POOL,
  SnarksWorkPoolActions,
  SnarksWorkPoolClose,
  SnarksWorkPoolGetWorkPool,
  SnarksWorkPoolGetWorkPoolDetail,
  SnarksWorkPoolSetActiveWorkPool,
} from '@snarks/work-pool/snarks-work-pool.actions';
import { SnarksWorkPoolService } from '@snarks/work-pool/snarks-work-pool.service';
import { WorkPool } from '@shared/types/snarks/work-pool/work-pool.type';
import { WorkPoolSpecs } from '@shared/types/snarks/work-pool/work-pool-specs.type';
import { WorkPoolDetail } from '@shared/types/snarks/work-pool/work-pool-detail.type';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';

@Injectable({
  providedIn: 'root',
})
export class SnarksWorkPoolEffects extends MinaRustBaseEffect<SnarksWorkPoolActions> {

  readonly getWorkPool$: Effect;
  readonly selectActiveWorkPool$: Effect;
  readonly getActiveWorkPoolDetail$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private snarksWorkPoolService: SnarksWorkPoolService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getWorkPool$ = createEffect(() => this.actions$.pipe(
      ofType(SNARKS_WORK_POOL_GET_WORK_POOL, SNARKS_WORK_POOL_CLOSE),
      this.latestActionState<SnarksWorkPoolGetWorkPool | SnarksWorkPoolClose>(),
      filter(({ action }) => {
        return (action.type === SNARKS_WORK_POOL_GET_WORK_POOL && !this.pendingRequest) || action.type === SNARKS_WORK_POOL_CLOSE;
      }),
      tap(({ action }) => {
        if (action.type === SNARKS_WORK_POOL_GET_WORK_POOL) {
          this.pendingRequest = true;
        }
      }),
      switchMap(({ action, state }) =>
        action.type === SNARKS_WORK_POOL_CLOSE
          ? EMPTY
          : this.snarksWorkPoolService.getWorkPool(),
      ),
      map((payload: WorkPool[]) => ({ type: SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, SNARKS_WORK_POOL_GET_WORK_POOL_SUCCESS, []),
      tap(() => this.pendingRequest = false),
    ));

    this.selectActiveWorkPool$ = createEffect(() => this.actions$.pipe(
      ofType(SNARKS_WORK_POOL_SET_ACTIVE_WORK_POOL),
      this.latestActionState<SnarksWorkPoolSetActiveWorkPool>(),
      filter(({ action }) => hasValue(action.payload?.id)),
      map(({ action }) => ({ type: SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL, payload: action.payload })),
    ));

    this.getActiveWorkPoolDetail$ = createEffect(() => this.actions$.pipe(
      ofType(SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL, SNARKS_WORK_POOL_CLOSE),
      this.latestActionState<SnarksWorkPoolGetWorkPoolDetail | SnarksWorkPoolClose>(),
      filter(({ action }) => hasValue((action as SnarksWorkPoolGetWorkPoolDetail).payload?.id)),
      switchMap(({ action, state }) =>
        action.type === SNARKS_WORK_POOL_CLOSE
          ? EMPTY
          : forkJoin([
            this.snarksWorkPoolService.getWorkPoolSpecs(state.snarks.workPool.activeWorkPool.id),
            this.snarksWorkPoolService.getWorkPoolDetail(state.snarks.workPool.activeWorkPool.id),
          ]),
      ),
      map((payload: [WorkPoolSpecs, WorkPoolDetail]) => ({
        type: SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS,
        payload,
      })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, SNARKS_WORK_POOL_GET_WORK_POOL_DETAIL_SUCCESS, []),
    ));
  }
}
