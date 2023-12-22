import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { SEC_CONFIG_GRAY_PALETTE, SecDurationConfig, TableColumnList } from '@openmina/shared';
import { filter } from 'rxjs';
import { NodesLiveBlockEvent } from '@shared/types/nodes/live/nodes-live-block-event.type';
import { NodesLiveSortEvents } from '@nodes/live/nodes-live.actions';
import { selectNodesLiveFilteredEvents, selectNodesLiveSort } from '@nodes/live/nodes-live.state';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-nodes-live-events-table',
  templateUrl: './nodes-live-events-table.component.html',
  styleUrls: ['./nodes-live-events-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-minus-lg flex-column' },
})
export class NodesLiveEventsTableComponent extends MinaTableRustWrapper<NodesLiveBlockEvent> implements OnInit {

  readonly secConfig: SecDurationConfig = {
    color: true,
    onlySeconds: false,
    colors: SEC_CONFIG_GRAY_PALETTE,
    severe: 10,
    warn: 1,
    default: 0.01,
    undefinedAlternative: '-',
  };

  protected readonly tableHeads: TableColumnList<NodesLiveBlockEvent> = [
    { name: 'datetime', sort: 'timestamp' },
    { name: 'height' },
    { name: 'message' },
    { name: 'status' },
    { name: 'elapsed' },
  ];

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToNodesChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [165, 80, 160, 100, 80];
    this.table.sortClz = NodesLiveSortEvents;
    this.table.sortSelector = selectNodesLiveSort;
  }

  private listenToNodesChanges(): void {
    this.select(selectNodesLiveFilteredEvents, (events: NodesLiveBlockEvent[]) => {
      this.table.rows = events;
      this.table.detect();
    }, filter(Boolean));
  }
}
