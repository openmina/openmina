export interface NodesOverviewResync {
  kind: NodesOverviewResyncKindType;
  time: number;
  description?: string;
}

export enum NodesOverviewResyncKindType {
  RootLedgerChange = 'Root Ledger Change',
  FetchStagedLedgerError = 'Fetch Staged Ledger Error',
  EpochChange = 'Epoch Change',
  BestChainChange = 'Best Chain Change',
}

export interface NodesOverviewResyncUI extends NodesOverviewResync {
  timeAgo: string;
}
