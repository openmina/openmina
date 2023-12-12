import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { NodesLiveService } from '@nodes/live/nodes-live.service';
import {
  NODES_LIVE_CLOSE,
  NODES_LIVE_GET_NODES,
  NODES_LIVE_GET_NODES_SUCCESS,
  NodesLiveActions,
  NodesLiveClose,
  NodesLiveGetNodes
} from '@nodes/live/nodes-live.actions';
import { NodesLiveNode } from '@shared/types/nodes/live/nodes-live-node.type';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';

@Injectable({
  providedIn: 'root',
})
export class NodesLiveEffects extends MinaRustBaseEffect<NodesLiveActions> {

  readonly getNodes$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private nodesLiveService: NodesLiveService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getNodes$ = createEffect(() => this.actions$.pipe(
      ofType(NODES_LIVE_GET_NODES, NODES_LIVE_CLOSE),
      this.latestActionState<NodesLiveGetNodes | NodesLiveClose>(),
      filter(({ action }) => (action as any).payload?.force || !this.pendingRequest),
      tap(({ action }) => {
        if (action.type === NODES_LIVE_GET_NODES) {
          this.pendingRequest = true;
        }
      }),
      switchMap(({ action }) =>
        action.type === NODES_LIVE_CLOSE
          ? EMPTY
          : this.nodesLiveService.getLiveNodeTips(),
      ),
      map((payload: NodesLiveNode[]) => ({ type: NODES_LIVE_GET_NODES_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, NODES_LIVE_GET_NODES_SUCCESS, { blocks: [], events: [] }),
      tap(() => this.pendingRequest = false),
    ));
  }
}
