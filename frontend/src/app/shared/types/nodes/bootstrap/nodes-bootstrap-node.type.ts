import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';

export interface NodesBootstrapNode extends NodesOverviewNode {
  index: number;
  height: number;
  globalSlot: number;
  appliedBlocksAvg: number;
  appliedBlocksMin: number;
  appliedBlocksMax: number;
  fetchedBlocksAvg: number;
  fetchedBlocksMin: number;
  fetchedBlocksMax: number;
}
