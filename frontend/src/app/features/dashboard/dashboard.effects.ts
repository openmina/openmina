import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { catchError, combineLatest, EMPTY, filter, forkJoin, map, mergeMap, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  DASHBOARD_CLOSE,
  DASHBOARD_GET_DATA,
  DASHBOARD_GET_DATA_SUCCESS,
  DASHBOARD_INIT,
  DashboardActions,
  DashboardClose,
  DashboardGetData,
} from '@dashboard/dashboard.actions';
import { DashboardService } from '@dashboard/dashboard.service';
import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';

@Injectable({
  providedIn: 'root',
})
export class DashboardEffects extends MinaRustBaseEffect<DashboardActions> {

  readonly init$: Effect;
  readonly getData$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private dashboardService: DashboardService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_INIT),
      map(() => ({ type: DASHBOARD_GET_DATA })),
    ));

    this.getData$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_GET_DATA, DASHBOARD_CLOSE),
      this.latestActionState<DashboardGetData | DashboardClose>(),
      filter(({ action }) => (action as DashboardGetData).payload?.force || !this.pendingRequest),
      tap(({ action }) => {
        if (action.type === DASHBOARD_GET_DATA) {
          this.pendingRequest = true;
        }
      }),
      switchMap(({ action, state }) =>
        action.type === DASHBOARD_CLOSE
          ? EMPTY
          : combineLatest([
            this.dashboardService.getPeers(),
            this.dashboardService.getTips({
              url: state.app.activeNode.url,
              name: state.app.activeNode.name,
            }),
            this.dashboardService.getRpcCalls(),
          ]).pipe(
            // tap((r) => {
            //   console.log('RESPONSE FROM COMBINATION', r);
            // }),
            // catchError((err) => {
            //   console.log('ERROR FROM COMBINATION', err);
            //   return EMPTY;
            // }),
          ),
      ),
      map((payload: [DashboardPeer[], NodesOverviewNode[], DashboardRpcStats]) => ({
        type: DASHBOARD_GET_DATA_SUCCESS, payload: { peers: payload[0], ledger: payload[1], rpcStats: payload[2] },
      })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, DASHBOARD_GET_DATA_SUCCESS, {
        peers: [],
        ledger: [],
        rpcStats: { peerResponses: [], stakingLedger: null, nextLedger: null, rootLedger: null },
      }),
      tap(() => this.pendingRequest = false),
    ));
  }
}
