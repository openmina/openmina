import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, forkJoin, map, switchMap, tap } from 'rxjs';
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
import { NodesOverviewService } from '@nodes/overview/nodes-overview.service';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { DashboardPeerRpcResponses, DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';

@Injectable({
  providedIn: 'root',
})
export class DashboardEffects extends MinaRustBaseEffect<DashboardActions> {

  readonly init$: Effect;
  readonly getData$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private dashboardService: DashboardService,
              private nodesOverviewService: NodesOverviewService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_INIT),
      map(() => ({ type: DASHBOARD_GET_DATA })),
    ));

    // !!! add to loading reducer as well when uncomment
    // this.getPeers$ = createEffect(() => this.actions$.pipe(
    //   ofType(DASHBOARD_GET_PEERS, DASHBOARD_CLOSE),
    //   this.latestActionState<DashboardGetPeers | DashboardClose>(),
    //   filter(() => !this.pendingRequest),
    //   tap(({ action }) => {
    //     if (action.type === DASHBOARD_GET_PEERS) {
    //       this.pendingRequest = true;
    //     }
    //   }),
    //   switchMap(({ action }) =>
    //     action.type === DASHBOARD_CLOSE
    //       ? EMPTY
    //       : this.dashboardService.getPeers(),
    //   ),
    //   map((payload: DashboardPeer[]) => ({ type: DASHBOARD_GET_PEERS_SUCCESS, payload })),
    //   catchErrorAndRepeat(MinaErrorType.GENERIC, DASHBOARD_GET_PEERS_SUCCESS, []),
    //   tap(() => this.pendingRequest = false),
    // ));

    this.getData$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_GET_DATA, DASHBOARD_CLOSE),
      this.latestActionState<DashboardGetData | DashboardClose>(),
      filter(() => !this.pendingRequest),
      tap(({ action }) => {
        if (action.type === DASHBOARD_GET_DATA) {
          this.pendingRequest = true;
        }
      }),
      switchMap(({ action, state }) =>
        action.type === DASHBOARD_CLOSE
          ? EMPTY
          : forkJoin([
            this.dashboardService.getPeers(),
            this.nodesOverviewService.getNodeTips({
              url: state.app.activeNode.url,
              name: state.app.activeNode.name,
            }, '?limit=1', true),
            this.dashboardService.getRpcCalls(),
          ])
      ),
      map((payload: [DashboardPeer[], NodesOverviewNode[], DashboardRpcStats]) => ({
        type: DASHBOARD_GET_DATA_SUCCESS, payload: { peers: payload[0], ledger: payload[1], rpcStats: payload[2] },
      })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, DASHBOARD_GET_DATA_SUCCESS, {
        peers: [],
        ledger: [],
        rpcStats: { peerResponses: [], stakingLedger: null, nextLedger: null, rootLedger: null }
      }),
      tap(() => this.pendingRequest = false),
    ));
  }
}
