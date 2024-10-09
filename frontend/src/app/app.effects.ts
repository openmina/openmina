import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { createNonDispatchableEffect, Effect, NonDispatchableEffect, removeParamsFromURL } from '@openmina/shared';
import { filter, from, map, switchMap, tap } from 'rxjs';
import { AppActions } from '@app/app.actions';
import { Router } from '@angular/router';
import { FeatureType, MinaNode } from '@shared/types/core/environment/mina-env.type';
import { AppService } from '@app/app.service';
import { getFirstFeature, isFeatureEnabled } from '@shared/constants/config';
import { RustService } from '@core/services/rust.service';
import { BaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import { WebNodeService } from '@core/services/web-node.service';
import { catchErrorAndRepeat2 } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { AppNodeStatus } from '@shared/types/app/app-node-details.type';

const INIT_EFFECTS = '@ngrx/effects/init';

@Injectable({
  providedIn: 'root',
})
export class AppEffects extends BaseEffect {

  readonly initEffects$: Effect;
  readonly init$: Effect;
  readonly initSuccess$: NonDispatchableEffect;
  readonly onNodeChange$: Effect;
  readonly getNodeDetails$: Effect;

  private requestInProgress: boolean = false;

  constructor(private actions$: Actions,
              private appService: AppService,
              private rustNode: RustService,
              private router: Router,
              private webNodeService: WebNodeService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.initEffects$ = createEffect(() => this.actions$.pipe(
      ofType(INIT_EFFECTS),
      map(() => AppActions.init()),
    ));

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(AppActions.init),
      switchMap(() => this.appService.getNodes()),
      switchMap((nodes: MinaNode[]) => this.appService.getActiveNode(nodes).pipe(
        tap((activeNode: MinaNode) => this.rustNode.changeRustNode(activeNode)),
        map((activeNode: MinaNode) => ({ activeNode, nodes })),
      )),
      map((payload: { activeNode: MinaNode, nodes: MinaNode[] }) => AppActions.initSuccess(payload)),
    ));

    this.initSuccess$ = createNonDispatchableEffect(() => this.actions$.pipe(
      ofType(AppActions.initSuccess),
      this.latestActionState(),
      switchMap(({ state }) => {
        if (state.app.activeNode.isWebNode) {
          return this.webNodeService.loadWasm$().pipe(
            switchMap(() => this.webNodeService.startWasm$()),
          );
        }
        return from([]);
      }),
    ));

    this.onNodeChange$ = createNonDispatchableEffect(() => this.actions$.pipe(
      ofType(AppActions.changeActiveNode),
      this.latestActionState(),
      tap(({ state }) => {
        this.rustNode.changeRustNode(state.app.activeNode);
        const activePage = removeParamsFromURL(this.router.url.split('/')[1]) as FeatureType;
        this.router.navigate([], {
          queryParams: { node: state.app.activeNode.name },
          queryParamsHandling: 'merge',
        });
        if (!isFeatureEnabled(state.app.activeNode, activePage)) {
          this.router.navigate([getFirstFeature(state.app.activeNode)]);
        }
      }),
      switchMap(({ state }) => {
        if (state.app.activeNode.isWebNode) {
          return this.webNodeService.loadWasm$().pipe(
            switchMap(() => this.webNodeService.startWasm$()),
          );
        }
        return from([]);
      }),
      map(() => AppActions.getNodeDetails()),
    ));

    this.getNodeDetails$ = createEffect(() => this.actions$.pipe(
      ofType(AppActions.getNodeDetails),
      filter(() => !this.requestInProgress),
      tap(() => this.requestInProgress = true),
      switchMap(() => this.appService.getActiveNodeDetails()),
      tap(() => this.requestInProgress = false),
      map(details => AppActions.getNodeDetailsSuccess({ details })),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, AppActions.getNodeDetailsSuccess({
        details: {
          status: AppNodeStatus.OFFLINE,
          blockHeight: null,
          blockTime: null,
          peers: 0,
          download: 0,
          upload: 0,
          transactions: 0,
          snarks: 0,
        },
      })),
    ));
  }
}
