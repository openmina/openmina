import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  DASHBOARD_CLOSE,
  DASHBOARD_GET_PEERS,
  DASHBOARD_GET_PEERS_SUCCESS, DASHBOARD_INIT,
  DashboardActions,
  DashboardClose,
  DashboardGetPeers,
} from '@dashboard/dashboard.actions';
import { DashboardService } from '@dashboard/dashboard.service';
import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';

@Injectable({
  providedIn: 'root',
})
export class DashboardEffects extends MinaRustBaseEffect<DashboardActions> {

  readonly init$: Effect;
  readonly getPeers$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private dashboardService: DashboardService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_INIT),
      map(() => ({ type: DASHBOARD_GET_PEERS })),
    ));

    this.getPeers$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_GET_PEERS, DASHBOARD_CLOSE),
      this.latestActionState<DashboardGetPeers | DashboardClose>(),
      filter(() => !this.pendingRequest),
      tap(({ action }) => {
        if (action.type === DASHBOARD_GET_PEERS) {
          this.pendingRequest = true;
        }
      }),
      switchMap(({ action }) =>
        action.type === DASHBOARD_CLOSE
          ? EMPTY
          : this.dashboardService.getPeers(),
      ),
      map((payload: DashboardPeer[]) => ({ type: DASHBOARD_GET_PEERS_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, DASHBOARD_GET_PEERS_SUCCESS, []),
      tap(() => this.pendingRequest = false),
    ));
  }
}
