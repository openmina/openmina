export interface NodesOverviewBlock {
  globalSlot: number;
  height: number;
  hash: string;
  predHash: string;
  status: NodesOverviewNodeBlockStatus;
  fetchStart: number;
  fetchEnd: number;
  applyStart: number;
  applyEnd: number;
  fetchDuration: number;
  applyDuration: number;
  isBestTip?: boolean;
}

export enum NodesOverviewNodeBlockStatus {
  APPLIED = 'Applied',
  APPLYING = 'Applying',
  FETCHED = 'Fetched',
  FETCHING = 'Fetching',
  MISSING = 'Missing',
}
