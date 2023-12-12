import { Injectable } from '@angular/core';
import { NodesOverviewService } from '@nodes/overview/nodes-overview.service';
import { map, Observable } from 'rxjs';
import { NodesOverviewNode, NodesOverviewNodeKindType } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { NodesBootstrapNode } from '@shared/types/nodes/bootstrap/nodes-bootstrap-node.type';
import { NodesOverviewNodeBlockStatus } from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import { hasValue } from '@openmina/shared';
import { RustService } from '@core/services/rust.service';

@Injectable({
  providedIn: 'root',
})
export class NodesBootstrapService {

  constructor(private nodesOverviewService: NodesOverviewService,
              private rust: RustService) { }

  getBootstrapNodeTips(): Observable<NodesBootstrapNode[]> {
    return this.nodesOverviewService.getNodeTips({ url: this.rust.URL, name: this.rust.name }).pipe(
      map((nodes: NodesOverviewNode[]) => nodes
        .filter(n => n.kind !== NodesOverviewNodeKindType.OFFLINE)
        .map((node: NodesOverviewNode, index: number) => {
          const appliedBlocks = node.blocks.filter((block) => block.status === NodesOverviewNodeBlockStatus.APPLIED && hasValue(block.applyStart));
          const fetchedBlocks = node.blocks.filter((block) => block.status === NodesOverviewNodeBlockStatus.FETCHED && hasValue(block.fetchStart));
          const applyBlocksDurations = appliedBlocks.map((block) => block.applyDuration);
          const fetchBlocksDurations = fetchedBlocks.map((block) => block.fetchDuration);
          return ({
            ...node,
            index,
            fetchedBlocks: fetchedBlocks.length,
            appliedBlocksAvg: appliedBlocks.reduce((sum, block) => sum + block.applyDuration, 0) / (appliedBlocks.length || 1),
            appliedBlocksMin: applyBlocksDurations.length ? Math.min(...applyBlocksDurations) : 0,
            appliedBlocksMax: Math.max(...appliedBlocks.map((block) => block.applyDuration), 0),
            appliedBlocks: appliedBlocks.length,
            fetchedBlocksAvg: fetchedBlocks.reduce((sum, block) => sum + block.fetchDuration, 0) / (fetchedBlocks.length || 1),
            fetchedBlocksMin: fetchBlocksDurations.length ? Math.min(...fetchBlocksDurations) : 0,
            fetchedBlocksMax: Math.max(...fetchedBlocks.map((block) => block.fetchDuration), 0),
            globalSlot: node.blocks[0]?.globalSlot,
            height: node.blocks[0]?.height,
          } as NodesBootstrapNode);
        })),
    );
  }
}
