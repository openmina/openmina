import { Injectable } from '@angular/core';
import { Effect } from '@openmina/shared';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Store } from '@ngrx/store';
import { MinaState, selectMinaState } from '@app/app.setup';
import { EMPTY, filter, map, switchMap } from 'rxjs';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import {
  TESTING_TOOL_SCENARIOS_ADD_STEP,
  TESTING_TOOL_SCENARIOS_CLOSE,
  TESTING_TOOL_SCENARIOS_CREATE_CLUSTER,
  TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS,
  TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS,
  TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS,
  TESTING_TOOL_SCENARIOS_GET_SCENARIO,
  TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS, TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO,
  TESTING_TOOL_SCENARIOS_START_SCENARIO,
  TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS,
  TestingToolScenariosActions, TestingToolScenariosAddStep,
  TestingToolScenariosClose,
  TestingToolScenariosCreateCluster,
  TestingToolScenariosGetPendingEvents,
  TestingToolScenariosGetScenario, TestingToolScenariosGetScenarioSuccess, TestingToolScenariosReloadScenario,
  TestingToolScenariosStartScenario,
} from '@testing-tool/scenarios/testing-tool-scenarios.actions';
import { TestingToolScenariosService } from '@testing-tool/scenarios/testing-tool-scenarios.service';
import { TestingToolScenario } from '@shared/types/testing-tool/scenarios/testing-tool-scenario.type';
import { TestingToolScenarioEvent } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-event.type';

@Injectable({
  providedIn: 'root',
})
export class TestingToolScenariosEffects extends MinaRustBaseEffect<TestingToolScenariosActions> {

  readonly getScenario$: Effect;
  readonly reloadScenario$: Effect;
  readonly reloadScenarioSuccess$: Effect;
  readonly addStep$: Effect;
  readonly createCluster$: Effect;
  readonly createClusterSuccess$: Effect;
  readonly startScenario$: Effect;
  readonly startScenarioSuccess$: Effect;
  readonly getPendingEvents$: Effect;

  constructor(private actions$: Actions,
              private testingToolScenariosService: TestingToolScenariosService,
              store: Store<MinaState>) {
    super(store, selectMinaState);

    this.getScenario$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_GET_SCENARIO, TESTING_TOOL_SCENARIOS_CLOSE),
      this.latestActionState<TestingToolScenariosGetScenario | TestingToolScenariosClose>(),
      switchMap(({ action }) =>
        action.type === TESTING_TOOL_SCENARIOS_CLOSE
          ? EMPTY
          : this.testingToolScenariosService.getScenario(),
      ),
      map((payload: TestingToolScenario) => ({ type: TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS, {}),
    ));

    this.addStep$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_ADD_STEP),
      this.latestActionState<TestingToolScenariosAddStep>(),
      switchMap(({ action, state }) =>
        this.testingToolScenariosService.addStep(state.testingTool.scenarios.scenario.info.id, action.payload.step),
      ),
      map(() => ({ type: TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, null),
    ));

    this.reloadScenario$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO),
      this.latestActionState<TestingToolScenariosReloadScenario>(),
      switchMap(({ state }) =>
        this.testingToolScenariosService.reloadScenario(state.testingTool.scenarios.clusterId, state.testingTool.scenarios.scenario.info.id),
      ),
      map((payload: TestingToolScenario) => ({ type: TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS, {}),
    ));

    this.reloadScenarioSuccess$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS),
      this.latestActionState<TestingToolScenariosGetScenarioSuccess>(),
      filter(({ state }) => state.testingTool.scenarios.runScenario),
      map(() => ({ type: TESTING_TOOL_SCENARIOS_START_SCENARIO })),
    ));

    this.createCluster$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_CREATE_CLUSTER),
      this.latestActionState<TestingToolScenariosCreateCluster>(),
      switchMap(({ action, state }) =>
        this.testingToolScenariosService.createCluster(state.testingTool.scenarios.scenario.info.id),
      ),
      map((payload: string) => ({ type: TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS, payload })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS),
    ));

    this.createClusterSuccess$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS),
      map(() => ({ type: TESTING_TOOL_SCENARIOS_START_SCENARIO })),
    ));

    this.startScenario$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_START_SCENARIO),
      this.latestActionState<TestingToolScenariosStartScenario>(),
      switchMap(({ state }) =>
        this.testingToolScenariosService.startScenario(state.testingTool.scenarios.clusterId),
      ),
      map(() => ({ type: TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS),
    ));

    this.startScenarioSuccess$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS),
      map(() => ({ type: TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS })),
    ));

    this.getPendingEvents$ = createEffect(() => this.actions$.pipe(
      ofType(TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS),
      this.latestActionState<TestingToolScenariosGetPendingEvents>(),
      switchMap(({ state }) =>
        this.testingToolScenariosService.getPendingEvents(state.testingTool.scenarios.clusterId),
      ),
      map((payload: TestingToolScenarioEvent[]) => ({
        type: TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS,
        payload,
      })),
    ));
  }
}
