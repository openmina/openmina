import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, map, switchMap, tap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import {
  NODES_OVERVIEW_CLOSE,
  NODES_OVERVIEW_GET_NODES,
  NODES_OVERVIEW_GET_NODES_SUCCESS,
  NodesOverviewActions,
  NodesOverviewClose,
  NodesOverviewGetNodes,
} from '@nodes/overview/nodes-overview.actions';
import { NodesOverviewService } from '@nodes/overview/nodes-overview.service';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';

@Injectable({
  providedIn: 'root',
})
export class NodesOverviewEffects extends MinaRustBaseEffect<NodesOverviewActions> {

  readonly getNodes$: Effect;

  private pendingRequest: boolean;

  constructor(private actions$: Actions,
              private nodesOverviewService: NodesOverviewService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getNodes$ = createEffect(() => this.actions$.pipe(
      ofType(NODES_OVERVIEW_GET_NODES, NODES_OVERVIEW_CLOSE),
      this.latestActionState<NodesOverviewGetNodes | NodesOverviewClose>(),
      filter(() => !this.pendingRequest),
      tap(({ action }) => {
        if (action.type === NODES_OVERVIEW_GET_NODES) {
          this.pendingRequest = true;
        }
      }),
      switchMap(({ action, state }) =>
        action.type === NODES_OVERVIEW_CLOSE
          ? EMPTY
          : this.nodesOverviewService.getNodes(state.app.nodes),
      ),
      map((payload: NodesOverviewNode[]) => ({ type: NODES_OVERVIEW_GET_NODES_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, NODES_OVERVIEW_GET_NODES_SUCCESS, []),
      tap(() => this.pendingRequest = false),
    ));
  }
}
