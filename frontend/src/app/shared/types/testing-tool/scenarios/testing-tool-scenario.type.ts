import { TestingToolScenarioStep } from '@shared/types/testing-tool/scenarios/testing-tool-scenario-step.type';

export interface TestingToolScenario {
  info: {
    id: string;
    description: string;
    parent_id: string;
    nodes: TestingToolScenarioNode[];
  };
  steps: TestingToolScenarioStep[];
}

export interface TestingToolScenarioNode {
  kind: string;
  chain_id: string;
  initial_time: number;
}
