import { Injectable } from '@angular/core';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { Effect } from '@openmina/shared';
import { EMPTY, map, switchMap } from 'rxjs';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  DASHBOARD_SPLITS_CLOSE,
  DASHBOARD_SPLITS_GET_SPLITS,
  DASHBOARD_SPLITS_GET_SPLITS_SUCCESS,
  DASHBOARD_SPLITS_MERGE_NODES, DASHBOARD_SPLITS_MERGE_NODES_SUCCESS,
  DASHBOARD_SPLITS_SPLIT_NODES,
  DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS,
  DashboardSplitsActions,
  DashboardSplitsClose,
  DashboardSplitsGetSplits,
  DashboardSplitsMergeNodes,
} from '@network/splits/dashboard-splits.actions';
import { DashboardSplitsService } from '@network/splits/dashboard-splits.service';
import { MinaState, selectMinaState } from '@app/app.setup';
import { DashboardSplits } from '@shared/types/network/splits/dashboard-splits.type';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { DashboardSplitsState } from '@network/splits/dashboard-splits.state';

@Injectable({
  providedIn: 'root',
})
export class DashboardSplitsEffects extends MinaRustBaseEffect<DashboardSplitsActions> {

  readonly getSplits$: Effect;
  readonly splitNodes$: Effect;
  readonly mergeNodes$: Effect;

  constructor(private actions$: Actions,
              private splitService: DashboardSplitsService,
              store: Store<MinaState>) {

    super(store, selectMinaState);

    this.getSplits$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_SPLITS_GET_SPLITS, DASHBOARD_SPLITS_CLOSE),
      this.latestActionState<DashboardSplitsGetSplits | DashboardSplitsClose>(),
      switchMap(({ action }) =>
        action.type === DASHBOARD_SPLITS_CLOSE
          ? EMPTY
          : this.splitService.getPeers(),
      ),
      map((payload: DashboardSplits) => ({ type: DASHBOARD_SPLITS_GET_SPLITS_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.RUST, DASHBOARD_SPLITS_GET_SPLITS_SUCCESS, { peers: [], links: [] }),
    ));

    this.splitNodes$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_SPLITS_SPLIT_NODES),
      this.latestStateSlice<DashboardSplitsState, DashboardSplitsMergeNodes>('network.splits'),
      switchMap(state => this.splitService.splitNodes(state.peers)),
      map(() => ({ type: DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS })),
      catchErrorAndRepeat(MinaErrorType.RUST, DASHBOARD_SPLITS_SPLIT_NODES_SUCCESS),
    ));

    this.mergeNodes$ = createEffect(() => this.actions$.pipe(
      ofType(DASHBOARD_SPLITS_MERGE_NODES),
      this.latestStateSlice<DashboardSplitsState, DashboardSplitsMergeNodes>('network.splits'),
      switchMap(state => this.splitService.mergeNodes(state.peers)),
      map(() => ({ type: DASHBOARD_SPLITS_MERGE_NODES_SUCCESS })),
      catchErrorAndRepeat(MinaErrorType.RUST, DASHBOARD_SPLITS_MERGE_NODES_SUCCESS),
    ));
  }
}
