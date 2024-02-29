import { Injectable } from '@angular/core';
import { NodesOverviewService } from '@nodes/overview/nodes-overview.service';
import { map, Observable } from 'rxjs';
import { NodesOverviewNode, NodesOverviewNodeKindType } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { NodesLiveNode } from '@shared/types/nodes/live/nodes-live-node.type';
import { NodesLiveBlockEvent } from '@shared/types/nodes/live/nodes-live-block-event.type';
import {
  NodesOverviewBlock,
  NodesOverviewNodeBlockStatus
} from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import { ONE_MILLION, toReadableDate } from '@openmina/shared';
import { RustService } from '@core/services/rust.service';

@Injectable({
  providedIn: 'root',
})
export class NodesLiveService {

  constructor(private nodesOverviewService: NodesOverviewService,
              private rust: RustService) { }

  getLiveNodeTips(): Observable<NodesLiveNode[]> {
    return this.nodesOverviewService.getNodeTips({ url: this.rust.URL, name: this.rust.name }).pipe(
      map((nodes: NodesOverviewNode[]) => nodes
        .filter(n => n.kind !== NodesOverviewNodeKindType.OFFLINE)
        .reverse()
        .map((node: NodesOverviewNode, index: number) => {
          return ({
            ...node,
            index,
            events: this.getEvents(node),
          } as NodesLiveNode);
        })),
    );
  }

  private getEvents(node: NodesOverviewNode): NodesLiveBlockEvent[] {
    const events: NodesLiveBlockEvent[] = [];
    const STARTED = 'Started';
    const COMPLETED = 'Completed';

    node.blocks.forEach((block: NodesOverviewBlock, index: number) => {
      const isBestTip = index === 0;
      if (block.fetchStart) {
        const event = {} as NodesLiveBlockEvent;
        event.height = block.height;
        event.message = NodesOverviewNodeBlockStatus.FETCHING;
        event.timestamp = block.fetchStart;
        event.datetime = toReadableDate(block.fetchStart / ONE_MILLION);
        event.status = STARTED;
        event.isBestTip = isBestTip;
        events.push(event);
      }
      if (block.fetchEnd) {
        const event = {} as NodesLiveBlockEvent;
        event.height = block.height;
        event.message = NodesOverviewNodeBlockStatus.FETCHED;
        event.timestamp = block.fetchEnd;
        event.datetime = toReadableDate(block.fetchEnd / ONE_MILLION);
        event.elapsed = block.fetchDuration;
        event.status = COMPLETED;
        event.isBestTip = isBestTip;
        events.push(event);
      }
      if (block.applyStart) {
        const event = {} as NodesLiveBlockEvent;
        event.height = block.height;
        event.message = NodesOverviewNodeBlockStatus.APPLYING;
        event.timestamp = block.applyStart;
        event.datetime = toReadableDate(block.applyStart / ONE_MILLION);
        event.status = STARTED;
        event.isBestTip = isBestTip;
        events.push(event);
      }
      if (block.applyEnd) {
        const event = {} as NodesLiveBlockEvent;
        event.height = block.height;
        event.message = NodesOverviewNodeBlockStatus.APPLIED;
        event.timestamp = block.applyEnd;
        event.datetime = toReadableDate(block.applyEnd / ONE_MILLION);
        event.elapsed = block.applyDuration;
        event.status = COMPLETED;
        event.isBestTip = isBestTip;
        events.push(event);
      }
    });

    return events;
  }
}
