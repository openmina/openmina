export interface WebNodeDemoLoadingStep {
  name: string;
  loaded: boolean;
  attempt?: number;
  step: number;
}
