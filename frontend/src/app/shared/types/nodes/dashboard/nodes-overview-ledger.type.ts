export interface NodesOverviewLedger {
  root?: NodesOverviewLedgerStep;
  stakingEpoch?: NodesOverviewLedgerStep;
  nextEpoch?: NodesOverviewLedgerStep;
}

export interface NodesOverviewLedgerStep {
  state: NodesOverviewLedgerStepState;
  snarked: NodesOverviewLedgerStepSnarked;
  staged: NodesOverviewLedgerStepStaged;
  totalTime: number;
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

export interface NodesOverviewLedgerStepStaged {
  fetchPartsStart: number;
  fetchPartsEnd: number;
  reconstructStart: number;
  reconstructEnd: number;
  fetchPartsDuration: number;
  reconstructDuration: number;
}
