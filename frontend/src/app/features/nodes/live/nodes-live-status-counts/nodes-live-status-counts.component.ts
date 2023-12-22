import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectNodesLiveActiveNode } from '@nodes/live/nodes-live.state';
import { NodesLiveNode } from '@shared/types/nodes/live/nodes-live-node.type';

@Component({
  selector: 'mina-nodes-live-status-counts',
  templateUrl: './nodes-live-status-counts.component.html',
  styleUrls: ['./nodes-live-status-counts.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-lg fx-row-vert-cent ml-10 mr-10' },
})
export class NodesLiveStatusCountsComponent extends StoreDispatcher implements OnInit {

  missing: number = 0;
  fetching: number = 0;
  fetched: number = 0;
  applying: number = 0;
  applied: number = 0;

  ngOnInit(): void {
    this.listenToBestTip();
  }

  private listenToBestTip(): void {
    this.select(selectNodesLiveActiveNode, (node: NodesLiveNode) => {
      this.missing = node?.missingBlocks;
      this.fetching = node?.fetchingBlocks;
      this.fetched = node?.fetchedBlocks;
      this.applying = node?.applyingBlocks;
      this.applied = node?.appliedBlocks;
      this.detect();
    });
  }
}
