import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { EMPTY, filter, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  NETWORK_BOOTSTRAP_STATS_CLOSE,
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS,
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS,
  NETWORK_BOOTSTRAP_STATS_INIT,
  NetworkBootstrapStatsActions,
  NetworkBootstrapStatsClose,
  NetworkBootstrapStatsGetBootstrapStats,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';
import { NetworkBootstrapStatsService } from '@network/bootstrap-stats/network-bootstrap-stats.service';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

@Injectable({
  providedIn: 'root',
})
export class NetworkBootstrapStatsEffects extends MinaRustBaseEffect<NetworkBootstrapStatsActions> {

  readonly init$: Effect;
  readonly getBootstrapStats$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private service: NetworkBootstrapStatsService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(NETWORK_BOOTSTRAP_STATS_INIT),
      map(() => ({ type: NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS })),
    ));

    this.getBootstrapStats$ = createEffect(() => this.actions$.pipe(
      ofType(NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS, NETWORK_BOOTSTRAP_STATS_CLOSE),
      this.latestActionState<NetworkBootstrapStatsGetBootstrapStats | NetworkBootstrapStatsClose>(),
      filter(({ action }) => action.type === NETWORK_BOOTSTRAP_STATS_CLOSE || !this.pendingRequest),
      tap(({ action }) => {
        this.pendingRequest = action.type === NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS;
      }),
      switchMap(({ action }) =>
        action.type === NETWORK_BOOTSTRAP_STATS_CLOSE
          ? EMPTY
          : this.service.getDhtBootstrapStats(),
      ),
      map((payload: NetworkBootstrapStatsRequest[]) => ({
        type: NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS,
        payload,
      })),
      tap(() => this.pendingRequest = false),
      catchErrorAndRepeat(MinaErrorType.RUST, NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS, []),
    ));

  }
}
