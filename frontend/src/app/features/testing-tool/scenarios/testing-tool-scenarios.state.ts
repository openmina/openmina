import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { TestingToolScenario } from '@shared/types/testing-tool/scenarios/testing-tool-scenario.type';
import { TestingToolScenarioEvent } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-event.type';
import { selectTestingToolScenariosState } from '@testing-tool/testing-tool.state';

export interface TestingToolScenariosState {
  scenario: TestingToolScenario;
  pendingEvents: TestingToolScenarioEvent[];
  clusterId: string;
  scenarioIsRunning: boolean;
  scenarioHasRun: boolean;
  runScenario: boolean;
}

const select = <T>(selector: (state: TestingToolScenariosState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectTestingToolScenariosState,
  selector,
);

export const selectTestingToolScenariosScenario = select((state: TestingToolScenariosState): TestingToolScenario => state.scenario);
export const selectTestingToolScenariosPendingEvents = select((state: TestingToolScenariosState): TestingToolScenarioEvent[] => state.pendingEvents);
export const selectTestingToolScenariosClusterId = select((state: TestingToolScenariosState): string => state.clusterId);
export const selectTestingToolScenariosScenarioIsRunning = select((state: TestingToolScenariosState): boolean => state.scenarioIsRunning);
export const selectTestingToolScenariosScenarioHasRun = select((state: TestingToolScenariosState): boolean => state.scenarioHasRun);
