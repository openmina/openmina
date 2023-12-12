import { MinaState } from '@app/app.setup';
import { createFeatureSelector, createSelector, MemoizedSelector } from '@ngrx/store';
import { TestingToolScenariosState } from '@testing-tool/scenarios/testing-tool-scenarios.state';

export interface TestingToolState {
  scenarios: TestingToolScenariosState;
}

const select = <T>(selector: (state: TestingToolState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectTestingToolState,
  selector,
);

export const selectTestingToolState = createFeatureSelector<TestingToolState>('testingTool');
export const selectTestingToolScenariosState = select((state: TestingToolState): TestingToolScenariosState => state.scenarios);
