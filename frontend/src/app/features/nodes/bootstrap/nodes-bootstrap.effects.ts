import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import {
  NODES_BOOTSTRAP_CLOSE,
  NODES_BOOTSTRAP_GET_NODES,
  NODES_BOOTSTRAP_GET_NODES_SUCCESS,
  NodesBootstrapActions,
  NodesBootstrapClose,
  NodesBootstrapGetNodes,
} from '@nodes/bootstrap/nodes-bootstrap.actions';
import { NodesBootstrapService } from '@nodes/bootstrap/nodes-bootstrap.service';
import { NodesBootstrapNode } from '@shared/types/nodes/bootstrap/nodes-bootstrap-node.type';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';

@Injectable({
  providedIn: 'root',
})
export class NodesBootstrapEffects extends MinaRustBaseEffect<NodesBootstrapActions> {

  readonly getNodes$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private nodesBootstrapService: NodesBootstrapService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getNodes$ = createEffect(() => this.actions$.pipe(
      ofType(NODES_BOOTSTRAP_GET_NODES, NODES_BOOTSTRAP_CLOSE),
      this.latestActionState<NodesBootstrapGetNodes | NodesBootstrapClose>(),
      filter(({ action }) => (action as any).payload?.force || !this.pendingRequest),
      tap(({ action }) => {
        if (action.type === NODES_BOOTSTRAP_GET_NODES) {
          this.pendingRequest = true;
        }
      }),
      switchMap(({ action, state }) =>
        action.type === NODES_BOOTSTRAP_CLOSE
          ? EMPTY
          : this.nodesBootstrapService.getBootstrapNodeTips(),
      ),
      map((payload: NodesBootstrapNode[]) => ({ type: NODES_BOOTSTRAP_GET_NODES_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, NODES_BOOTSTRAP_GET_NODES_SUCCESS, { blocks: [] }),
      tap(() => this.pendingRequest = false),
    ));
  }
}
