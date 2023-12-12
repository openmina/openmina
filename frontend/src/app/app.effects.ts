import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { createNonDispatchableEffect, Effect, removeParamsFromURL } from '@openmina/shared';
import { map, switchMap, tap } from 'rxjs';
import {
  APP_CHANGE_ACTIVE_NODE,
  APP_INIT,
  APP_INIT_SUCCESS,
  AppActions,
  AppChangeActiveNode,
} from '@app/app.actions';
import { Router } from '@angular/router';
import { FeatureType, MinaNode } from '@shared/types/core/environment/mina-env.type';
import { AppService } from '@app/app.service';
import { getFirstFeature, isFeatureEnabled } from '@shared/constants/config';
import { RustService } from '@core/services/rust.service';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';

const INIT_EFFECTS = '@ngrx/effects/init';

@Injectable({
  providedIn: 'root',
})
export class AppEffects extends MinaRustBaseEffect<AppActions> {

  readonly initEffects$: Effect;
  readonly init$: Effect;
  readonly onNodeChange$: Effect;

  constructor(private actions$: Actions,
              private appService: AppService,
              private rustNode: RustService,
              private router: Router,
              store: Store<MinaState>) {

    super(store, selectMinaState);

    this.initEffects$ = createEffect(() => this.actions$.pipe(
      ofType(INIT_EFFECTS),
      map(() => ({ type: APP_INIT })),
    ));

    this.init$ = createEffect(() => this.actions$.pipe(
      ofType(APP_INIT),
      switchMap(() => this.appService.getNodes()),
      switchMap((nodes: MinaNode[]) => this.appService.getActiveNode(nodes).pipe(
        tap((activeNode: MinaNode) => this.rustNode.changeRustNode(activeNode)),
        map((activeNode: MinaNode) => ({ activeNode, nodes })),
      )),
      map((payload: { activeNode: MinaNode, nodes: MinaNode[] }) => ({ type: APP_INIT_SUCCESS, payload })),
    ));

    this.onNodeChange$ = createNonDispatchableEffect(() => this.actions$.pipe(
      ofType(APP_CHANGE_ACTIVE_NODE),
      this.latestActionState<AppChangeActiveNode>(),
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
    ));
  }
}
