import { NodesOverviewLedger } from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { NodesOverviewBlock } from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import { NodesOverviewResync } from '@shared/types/nodes/dashboard/nodes-overview-resync.type';

export interface NodesOverviewNode {
  name: string;
  kind: NodesOverviewNodeKindType;
  bestTipReceived: string;
  bestTipReceivedTimestamp: number;
  bestTip: string;
  height: number;
  globalSlot: number;
  appliedBlocks: number;
  applyingBlocks: number;
  missingBlocks: number;
  fetchingBlocks: number;
  fetchedBlocks: number;
  ledgers: NodesOverviewLedger;
  resyncs: NodesOverviewResync[];
  blocks: NodesOverviewBlock[];
}

export enum NodesOverviewNodeKindType {
  BOOTSTRAP = 'Bootstrap',
  CATCHUP = 'Catchup',
  SYNCED = 'Synced',
  PENDING = 'Pending',
  OFFLINE = 'Offline',
}
