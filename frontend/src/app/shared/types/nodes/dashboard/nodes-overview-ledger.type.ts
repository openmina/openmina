export interface NodesOverviewLedger {
  root: NodesOverviewRootLedgerStep;
  stakingEpoch: NodesOverviewLedgerEpochStep;
  nextEpoch: NodesOverviewLedgerEpochStep;
}

export interface NodesOverviewLedgerEpochStep {
  state: NodesOverviewLedgerStepState;
  snarked: NodesOverviewLedgerStepSnarked;
  totalTime: number;
}

export interface NodesOverviewRootLedgerStep extends NodesOverviewLedgerEpochStep {
  staged: NodesOverviewStagedLedgerStep;
  synced: number;
}

export enum NodesOverviewLedgerStepState {
  PENDING = 'pending',
  LOADING = 'loading',
  SUCCESS = 'success',
}

export interface NodesOverviewLedgerStepSnarked {
  fetchHashesStart: number;
  fetchHashesEnd: number;
  fetchAccountsStart: number;
  fetchAccountsEnd: number;
  fetchHashesDuration: number;
  fetchAccountsDuration: number;
}

export interface NodesOverviewStagedLedgerStep {
  fetchPartsStart: number;
  fetchPartsEnd: number;
  reconstructStart: number;
  reconstructEnd: number;
  fetchPartsDuration: number;
  reconstructDuration: number;
}
