import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { createNonDispatchableEffect, Effect, removeParamsFromURL } from '@openmina/shared';
import { filter, map, mergeMap, of, switchMap, tap } from 'rxjs';
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

@Injectable({
  providedIn: 'root',
})
export class AppEffects extends BaseEffect {

  readonly init$: Effect;
  readonly initSuccess$: Effect;
  readonly onNodeChange$: Effect;
  readonly getNodeDetails$: Effect;
  readonly getNodeEnvBuild$: Effect;

  private requestInProgress: boolean = false;

  constructor(private actions$: Actions,
              private appService: AppService,
              private rustNode: RustService,
              private router: Router,
              private webNodeService: WebNodeService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(AppActions.init),
      switchMap(() => this.appService.getNodes()),
      switchMap((nodes: MinaNode[]) => this.appService.getActiveNode(nodes).pipe(
        tap((activeNode: MinaNode) => this.rustNode.changeRustNode(activeNode)),
        map((activeNode: MinaNode) => ({ activeNode, nodes })),
      )),
      map((payload: { activeNode: MinaNode, nodes: MinaNode[] }) => AppActions.initSuccess(payload)),
    ));

    this.initSuccess$ = createEffect(() => this.actions$.pipe(
      ofType(AppActions.initSuccess),
      this.latestActionState(),
      switchMap(({ state }) => {
        if (state.app.activeNode.isWebNode) {
          return this.webNodeService.loadWasm$().pipe(
            switchMap(() => this.webNodeService.startWasm$()),
          );
        }
        return of({});
      }),
      map(() => AppActions.getNodeEnvBuild()),
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
        return of({});
      }),
      switchMap(() => [AppActions.getNodeDetails(), AppActions.getNodeEnvBuild()]),
    ));

    this.getNodeEnvBuild$ = createEffect(() => this.actions$.pipe(
      ofType(AppActions.getNodeEnvBuild),
      mergeMap(() => this.appService.getEnvBuild()),
      map(envBuild => AppActions.getNodeEnvBuildSuccess({ envBuild })),
      catchErrorAndRepeat2(MinaErrorType.RUST, AppActions.getNodeEnvBuildSuccess({ envBuild: undefined })),
    ));

    this.getNodeDetails$ = createEffect(() => this.actions$.pipe(
      ofType(AppActions.getNodeDetails),
      filter(() => !this.requestInProgress),
      tap(() => this.requestInProgress = true),
      switchMap(() => this.appService.getActiveNodeDetails()),
      map(details => AppActions.getNodeDetailsSuccess({ details })),
      catchErrorAndRepeat2(MinaErrorType.GENERIC, AppActions.getNodeDetailsSuccess({
        details: {
          status: AppNodeStatus.OFFLINE,
          blockHeight: null,
          blockTime: null,
          peersConnected: 0,
          peersDisconnected: 0,
          peersConnecting: 0,
          transactions: 0,
          snarks: 0,
          producingBlockAt: null,
          producingBlockGlobalSlot: null,
          producingBlockStatus: null,
        },
      })),
      tap(() => this.requestInProgress = false),
    ));
  }
}
