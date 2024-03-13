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
  NETWORK_NODE_DHT_GET_PEERS,
  NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
  NETWORK_NODE_DHT_INIT,
  NetworkNodeDhtActions
} from '@network/node-dht/network-node-dht.actions';
import { NetworkNodeDhtService } from '@network/node-dht/network-node-dht.service';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';

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
      ofType(NETWORK_NODE_DHT_GET_PEERS),
      filter(() => !this.pendingRequest),
      tap(() => this.pendingRequest = true),
      switchMap(() => this.nodeDhtService.getDhtPeers()),
      map((payload: { peers: NetworkNodeDHT[], thisKey: string }) => ({
        type: NETWORK_NODE_DHT_GET_PEERS_SUCCESS,
        payload
      })),
      tap(() => this.pendingRequest = false),
      catchErrorAndRepeat(MinaErrorType.RUST, NETWORK_NODE_DHT_GET_PEERS_SUCCESS, { peers: [], thisKey: '' }),
    ));

  }
}
