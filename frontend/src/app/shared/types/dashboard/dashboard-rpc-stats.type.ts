export interface DashboardRpcStats {
  peerResponses: DashboardPeerRpcResponses[];
  stakingLedger: DashboardLedgerStepStats;
  nextLedger: DashboardLedgerStepStats;
  snarkedRootLedger: DashboardLedgerStepStats;
  stagedRootLedger: DashboardLedgerStepStats;
}

export interface DashboardPeerRpcResponses {
  peerId: string;
  requestsMade: number;
}

export interface DashboardLedgerStepStats {
  fetched: number;
  estimation: number;
}
