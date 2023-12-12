import { FeatureAction } from '@openmina/shared';
import { TestingToolScenario } from '@shared/types/testing-tool/scenarios/testing-tool-scenario.type';
import { TestingToolScenarioStep } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-step.type';
import { TestingToolScenarioEvent } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-event.type';

enum TestingToolScenariosActionTypes {
  TESTING_TOOL_SCENARIOS_CLOSE = 'TESTING_TOOL_SCENARIOS_CLOSE',
  TESTING_TOOL_SCENARIOS_GET_SCENARIO = 'TESTING_TOOL_SCENARIOS_GET_SCENARIO',
  TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS = 'TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS',
  TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO = 'TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO',
  TESTING_TOOL_SCENARIOS_ADD_STEP = 'TESTING_TOOL_SCENARIOS_ADD_STEP',
  TESTING_TOOL_SCENARIOS_CREATE_CLUSTER = 'TESTING_TOOL_SCENARIOS_CREATE_CLUSTER',
  TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS = 'TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS',
  TESTING_TOOL_SCENARIOS_START_SCENARIO = 'TESTING_TOOL_SCENARIOS_START_SCENARIO',
  TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS = 'TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS',
  TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS = 'TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS',
  TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS = 'TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS',
}

export const TESTING_TOOL_SCENARIOS_CLOSE = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_CLOSE;
export const TESTING_TOOL_SCENARIOS_GET_SCENARIO = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_GET_SCENARIO;
export const TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS;
export const TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO;
export const TESTING_TOOL_SCENARIOS_ADD_STEP = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_ADD_STEP;
export const TESTING_TOOL_SCENARIOS_CREATE_CLUSTER = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_CREATE_CLUSTER;
export const TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS;
export const TESTING_TOOL_SCENARIOS_START_SCENARIO = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_START_SCENARIO;
export const TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS;
export const TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS;
export const TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS = TestingToolScenariosActionTypes.TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS;

export interface TestingToolScenariosAction extends FeatureAction<TestingToolScenariosActionTypes> {
  readonly type: TestingToolScenariosActionTypes;
}

export class TestingToolScenariosClose implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_CLOSE;
}

export class TestingToolScenariosGetScenario implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_GET_SCENARIO;

  constructor(public payload: string) { }
}

export class TestingToolScenariosGetScenarioSuccess implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS;

  constructor(public payload: TestingToolScenario) { }
}

export class TestingToolScenariosReloadScenario implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_RELOAD_SCENARIO;
}

export class TestingToolScenariosAddStep implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_ADD_STEP;

  constructor(public payload: { step: any, runScenario?: boolean }) { }
}

export class TestingToolScenariosCreateCluster implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_CREATE_CLUSTER;
}

export class TestingToolScenariosCreateClusterSuccess implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS;

  constructor(public payload: string) { }
}

export class TestingToolScenariosStartScenario implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_START_SCENARIO;
}

export class TestingToolScenariosStartScenarioSuccess implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS;
}

export class TestingToolScenariosGetPendingEvents implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS;
}

export class TestingToolScenariosGetPendingEventsSuccess implements TestingToolScenariosAction {
  readonly type = TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS;

  constructor(public payload: TestingToolScenarioEvent[]) { }
}

export type TestingToolScenariosActions =
  | TestingToolScenariosClose
  | TestingToolScenariosGetScenario
  | TestingToolScenariosGetScenarioSuccess
  | TestingToolScenariosReloadScenario
  | TestingToolScenariosAddStep
  | TestingToolScenariosCreateCluster
  | TestingToolScenariosCreateClusterSuccess
  | TestingToolScenariosStartScenario
  | TestingToolScenariosStartScenarioSuccess
  | TestingToolScenariosGetPendingEvents
  | TestingToolScenariosGetPendingEventsSuccess
  ;
