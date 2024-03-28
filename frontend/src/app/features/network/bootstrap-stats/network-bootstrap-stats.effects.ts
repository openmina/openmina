import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { filter, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { Store } from '@ngrx/store';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS,
  NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS,
  NETWORK_BOOTSTRAP_STATS_INIT,
  NetworkBootstrapStatsActions,
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
      ofType(NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS),
      filter(() => !this.pendingRequest),
      tap(() => this.pendingRequest = true),
      switchMap(() => this.service.getDhtBootstrapStats()),
      map((payload: NetworkBootstrapStatsRequest[]) => ({
        type: NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS,
        payload,
      })),
      tap(() => this.pendingRequest = false),
      //todo: review catch error payload
      catchErrorAndRepeat(MinaErrorType.RUST, NETWORK_BOOTSTRAP_STATS_GET_BOOTSTRAP_STATS_SUCCESS, {}),
    ));

  }
}
