import { NodesLiveBlockEvent } from '@shared/types/nodes/live/nodes-live-block-event.type';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';

export interface NodesLiveNode extends NodesOverviewNode {
  index: number;
  events: NodesLiveBlockEvent[];
}
