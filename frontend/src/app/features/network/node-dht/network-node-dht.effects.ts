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
  NETWORK_NODE_DHT_CLOSE,
  NETWORK_NODE_DHT_GET_PEERS,
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
  NETWORK_NODE_DHT_INIT,
  NetworkNodeDhtActions, NetworkNodeDhtClose, NetworkNodeDhtGetPeers,
} from '@network/node-dht/network-node-dht.actions';
import { NetworkNodeDhtService } from '@network/node-dht/network-node-dht.service';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';
import {
  DASHBOARD_SPLITS_CLOSE,
  DASHBOARD_SPLITS_GET_SPLITS, DashboardSplitsClose,
  DashboardSplitsGetSplits,
} from '@network/splits/dashboard-splits.actions';
import {
  NODES_LIVE_CLOSE,
  NODES_LIVE_GET_NODES,
  NodesLiveClose,
  NodesLiveGetNodes,
} from '@nodes/live/nodes-live.actions';

@Injectable({
  providedIn: 'root',
})
export class NetworkNodeDhtEffects extends MinaRustBaseEffect<NetworkNodeDhtActions> {

  readonly init$: Effect;
  readonly getPeers$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private nodeDhtService: NetworkNodeDhtService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(NETWORK_NODE_DHT_INIT),
      map(() => ({ type: NETWORK_NODE_DHT_GET_PEERS })),
    ));

    this.getPeers$ = createEffect(() => this.actions$.pipe(
      ofType(NETWORK_NODE_DHT_GET_PEERS, NETWORK_NODE_DHT_CLOSE),
      this.latestActionState<NetworkNodeDhtGetPeers | NetworkNodeDhtClose>(),
      filter(({ action }) => action.type === NETWORK_NODE_DHT_CLOSE || !this.pendingRequest),
      tap(({ action }) => {
        this.pendingRequest = action.type === NETWORK_NODE_DHT_GET_PEERS;
      }),
      switchMap(({ action }) =>
        action.type === NETWORK_NODE_DHT_CLOSE
          ? EMPTY
          : this.nodeDhtService.getDhtPeers(),
      ),
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
  }
}
