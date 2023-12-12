import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, map, switchMap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { StateActionsService } from '@state/actions/state-actions.service';
import {
  STATE_ACTIONS_CLOSE,
  STATE_ACTIONS_GET_ACTIONS,
  STATE_ACTIONS_GET_ACTIONS_SUCCESS,
  STATE_ACTIONS_GET_EARLIEST_SLOT,
  STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS,
  StateActionsActions,
  StateActionsClose,
  StateActionsGetActions,
  StateActionsGetEarliestSlot,
} from '@state/actions/state-actions.actions';
import { StateActionGroup } from '@shared/types/state/actions/state-action-group.type';
import { StateActionsStats } from '@shared/types/state/actions/state-actions-stats.type';
import { Routes } from '@shared/enums/routes.enum';
import { Router } from '@angular/router';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';

@Injectable({
  providedIn: 'root',
})
export class StateActionsEffects extends MinaRustBaseEffect<StateActionsActions> {

  readonly getActions$: Effect;
  readonly getEarliestSlot$: Effect;

  constructor(private router: Router,
              private actions$: Actions,
              private actionsService: StateActionsService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getEarliestSlot$ = createEffect(() => this.actions$.pipe(
      ofType(STATE_ACTIONS_GET_EARLIEST_SLOT, STATE_ACTIONS_CLOSE),
      this.latestActionState<StateActionsGetEarliestSlot | StateActionsClose>(),
      switchMap(({ action, state }) =>
        action.type === STATE_ACTIONS_CLOSE
          ? EMPTY
          : this.actionsService.getEarliestSlot().pipe(
            switchMap((payload: number) => {
              const actions: StateActionsActions[] = [{ type: STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS, payload }];
              if (state.state.actions.activeSlot === undefined || state.state.actions.activeSlot > payload) {
                this.router.navigate([Routes.STATE, Routes.ACTIONS, payload ?? ''], { queryParamsHandling: 'merge' });
                actions.push({ type: STATE_ACTIONS_GET_ACTIONS, payload: { slot: payload } });
              }
              return actions;
            }),
          ),
      ),
      catchErrorAndRepeat(MinaErrorType.GENERIC, STATE_ACTIONS_GET_EARLIEST_SLOT_SUCCESS, null),
    ));

    this.getActions$ = createEffect(() => this.actions$.pipe(
      ofType(STATE_ACTIONS_GET_ACTIONS, STATE_ACTIONS_CLOSE),
      this.latestActionState<StateActionsGetActions | StateActionsClose>(),
      switchMap(({ action, state }) =>
        action.type === STATE_ACTIONS_CLOSE
          ? EMPTY
          : this.actionsService.getActions(state.state.actions.activeSlot),
      ),
      map((payload: [StateActionsStats, StateActionGroup[]]) => ({ type: STATE_ACTIONS_GET_ACTIONS_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, STATE_ACTIONS_GET_ACTIONS_SUCCESS, []),
    ));
  }
}
