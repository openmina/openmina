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
  NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS,
  NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS,
  NETWORK_NODE_DHT_GET_PEERS,
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
  NETWORK_NODE_DHT_INIT,
  NetworkNodeDhtActions,
} from '@network/node-dht/network-node-dht.actions';
import { NetworkNodeDhtService } from '@network/node-dht/network-node-dht.service';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

@Injectable({
  providedIn: 'root',
})
export class NetworkNodeDhtEffects extends MinaRustBaseEffect<NetworkNodeDhtActions> {

  readonly init$: Effect;
  readonly getPeers$: Effect;
  readonly getBootstrapStats$: Effect;

  private pendingRequest: boolean;
  private pendingRequest2: boolean;

  constructor(private actions$: Actions,
              private nodeDhtService: NetworkNodeDhtService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(NETWORK_NODE_DHT_INIT),
      switchMap(() => [
        { type: NETWORK_NODE_DHT_GET_PEERS },
        { type: NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS },
      ]),
    ));

    this.getPeers$ = createEffect(() => this.actions$.pipe(
      ofType(NETWORK_NODE_DHT_GET_PEERS),
      filter(() => !this.pendingRequest),
      tap(() => this.pendingRequest = true),
      switchMap(() => this.nodeDhtService.getDhtPeers()),
      map((payload: { peers: NetworkNodeDhtPeer[], thisKey: string, buckets: NetworkNodeDhtBucket[] }) => ({
        type: NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
        payload,
      })),
      tap(() => this.pendingRequest = false),
      catchErrorAndRepeat(MinaErrorType.RUST, NETWORK_NODE_DHT_GET_PEERS_SUCCESS, {
        peers: [],
        thisKey: '',
        buckets: [],
      }),
    ));

    this.getBootstrapStats$ = createEffect(() => this.actions$.pipe(
      ofType(NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS),
      filter(() => !this.pendingRequest2),
      tap(() => this.pendingRequest2 = true),
      switchMap(() => this.nodeDhtService.getDhtBootstrapStats()),
      map((payload: NetworkBootstrapStatsRequest[]) => ({
        type: NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS,
        payload,
      })),
      tap(() => this.pendingRequest2 = false),
      //todo: review catch error payload
      catchErrorAndRepeat(MinaErrorType.RUST, NETWORK_NODE_DHT_GET_BOOTSTRAP_STATS_SUCCESS, {}),
    ));

  }
}
