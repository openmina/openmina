import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { Router } from '@angular/router';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  SCAN_STATE_CLOSE,
  SCAN_STATE_GET_BLOCK,
  SCAN_STATE_GET_BLOCK_SUCCESS,
  SCAN_STATE_INIT,
  ScanStateActions,
  ScanStateClose,
  ScanStateGetBlock
} from '@snarks/scan-state/scan-state.actions';
import { ScanStateService } from '@snarks/scan-state/scan-state.service';
import { ScanStateBlock } from '@shared/types/snarks/scan-state/scan-state-block.type';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Routes } from '@shared/enums/routes.enum';

@Injectable({
  providedIn: 'root',
})
export class ScanStateEffects extends MinaRustBaseEffect<ScanStateActions> {

  readonly getScanState$: Effect;

  private inProgressGettingHeight: number | string = null;

  constructor(private actions$: Actions,
              private router: Router,
              private scanStateService: ScanStateService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getScanState$ = createEffect(() => this.actions$.pipe(
      ofType(SCAN_STATE_GET_BLOCK, SCAN_STATE_CLOSE),
      this.latestActionState<ScanStateGetBlock | ScanStateClose>(),
      filter(({ action }) => {
        return (action.type === SCAN_STATE_GET_BLOCK && action.payload.heightOrHash !== this.inProgressGettingHeight) || action.type === SCAN_STATE_CLOSE;
      }),
      tap(({ action, state }) => {
        if (action.type === SCAN_STATE_GET_BLOCK) {
          this.inProgressGettingHeight = action.payload.heightOrHash;
          if (!state.snarks.scanState.stream) {
            store.dispatch({ type: SCAN_STATE_INIT });
          }
        }
      }),
      switchMap(({ action, state }) =>
        action.type === SCAN_STATE_CLOSE
          ? EMPTY
          : this.scanStateService.getScanState(action.payload.heightOrHash)
      ),
      map((payload: ScanStateBlock) => ({ type: SCAN_STATE_GET_BLOCK_SUCCESS, payload })),
      tap(({ payload }) => {
        if (!this.router.url.includes(payload.height.toString()) && !this.router.url.includes(payload.hash)) {
          this.router.navigate([Routes.SNARKS, Routes.SCAN_STATE, payload.height], { queryParamsHandling: 'merge' });
        }
      }),
      catchErrorAndRepeat(MinaErrorType.GENERIC, SCAN_STATE_GET_BLOCK_SUCCESS, { trees: [], workingSnarkers: [] }),
      tap(() => this.inProgressGettingHeight = null),
    ));
  }

}
