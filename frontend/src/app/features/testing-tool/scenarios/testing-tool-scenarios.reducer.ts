import { TestingToolScenariosState } from '@testing-tool/scenarios/testing-tool-scenarios.state';
import {
  TESTING_TOOL_SCENARIOS_ADD_STEP,
  TESTING_TOOL_SCENARIOS_CLOSE,
  TESTING_TOOL_SCENARIOS_CREATE_CLUSTER,
  TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS,
  TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS,
  TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS, TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS,
  TestingToolScenariosActions,
} from '@testing-tool/scenarios/testing-tool-scenarios.actions';

const initialState: TestingToolScenariosState = {
  scenario: undefined,
  pendingEvents: [],
  clusterId: undefined,
  scenarioIsRunning: false,
  scenarioHasRun: false,
  runScenario: false,
};

export function reducer(state: TestingToolScenariosState = initialState, action: TestingToolScenariosActions): TestingToolScenariosState {
  switch (action.type) {

    case TESTING_TOOL_SCENARIOS_GET_SCENARIO_SUCCESS: {
      console.log(action.payload);
      return {
        ...state,
        scenario: action.payload,
      };
    }

    case TESTING_TOOL_SCENARIOS_ADD_STEP: {
      return {
        ...state,
        runScenario: action.payload.runScenario,
      }
    }

    case TESTING_TOOL_SCENARIOS_CREATE_CLUSTER: {
      return {
        ...state,
        scenarioIsRunning: true,
      };
    }

    case TESTING_TOOL_SCENARIOS_CREATE_CLUSTER_SUCCESS: {
      return {
        ...state,
        clusterId: action.payload,
      };
    }

    case TESTING_TOOL_SCENARIOS_GET_PENDING_EVENTS_SUCCESS: {
      return {
        ...state,
        pendingEvents: action.payload,
        scenarioIsRunning: false,
      };
    }

    case TESTING_TOOL_SCENARIOS_START_SCENARIO_SUCCESS: {
      return {
        ...state,
        runScenario: false,
        scenarioHasRun: true,
      }
    }

    case TESTING_TOOL_SCENARIOS_CLOSE:
      return initialState;

    default:
      return state;
  }
}
