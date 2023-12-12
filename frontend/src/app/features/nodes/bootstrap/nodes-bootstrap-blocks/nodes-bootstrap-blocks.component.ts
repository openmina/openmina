import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NodesBootstrapNode } from '@shared/types/nodes/bootstrap/nodes-bootstrap-node.type';
import { selectNodesBootstrapActiveNode } from '@nodes/bootstrap/nodes-bootstrap.state';
import {
  NodesOverviewBlock,
  NodesOverviewNodeBlockStatus
} from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import { SEC_CONFIG_GRAY_PALETTE, SecDurationConfig, sort, SortDirection, TableSort } from '@openmina/shared';

@Component({
  selector: 'mina-nodes-bootstrap-blocks',
  templateUrl: './nodes-bootstrap-blocks.component.html',
  styleUrls: ['./nodes-bootstrap-blocks.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NodesBootstrapBlocksComponent extends StoreDispatcher implements OnInit {

  readonly secConfig: SecDurationConfig = { color: true, onlySeconds: false, colors: SEC_CONFIG_GRAY_PALETTE, severe: 10, warn: 1, default: 0.01, undefinedAlternative: '-'};
  activeNode: NodesBootstrapNode;
  fetchedBlocks: NodesOverviewBlock[] = [];
  appliedBlocks: NodesOverviewBlock[] = [];
  activeTab: number = 0;

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(selectNodesBootstrapActiveNode, (activeNode: NodesBootstrapNode) => {
      this.activeNode = activeNode;
      this.fetchedBlocks = sortBlocks(
        activeNode?.blocks.filter(b => b.status === NodesOverviewNodeBlockStatus.FETCHED || b.fetchDuration > 0) || [],
        { sortBy: 'fetchDuration', sortDirection: SortDirection.DSC },
      );
      this.appliedBlocks = sortBlocks(
        activeNode?.blocks.filter(b => b.status === NodesOverviewNodeBlockStatus.APPLIED) || [],
        { sortBy: 'applyDuration', sortDirection: SortDirection.DSC },
      );
      this.detect();
    });
  }

  selectTab(tab: number): void {
    this.activeTab = tab;
  }
}

function sortBlocks(blocks: NodesOverviewBlock[], tableSort: TableSort<NodesOverviewBlock>): NodesOverviewBlock[] {
  return sort<NodesOverviewBlock>(blocks, tableSort, ['hash']);
}
