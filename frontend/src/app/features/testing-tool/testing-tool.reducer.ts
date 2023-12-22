import { ActionReducer, combineReducers } from '@ngrx/store';

import * as fromScenarios from '@testing-tool/scenarios/testing-tool-scenarios.reducer';
import {
  TestingToolScenariosAction,
  TestingToolScenariosActions
} from '@testing-tool/scenarios/testing-tool-scenarios.actions';
import { TestingToolState } from '@testing-tool/testing-tool.state';

export type TestingToolActions =
  & TestingToolScenariosActions
  ;

export type TestingToolAction =
  & TestingToolScenariosAction
  ;

export const reducer: ActionReducer<TestingToolState, TestingToolActions> = combineReducers<TestingToolState, TestingToolActions>({
  scenarios: fromScenarios.reducer,
});
