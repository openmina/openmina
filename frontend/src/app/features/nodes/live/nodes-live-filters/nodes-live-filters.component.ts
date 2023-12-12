import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectNodesLiveFilters } from '@nodes/live/nodes-live.state';
import { NodesLiveToggleFilter } from '@nodes/live/nodes-live.actions';
import { NodesOverviewNodeBlockStatus } from '@shared/types/nodes/dashboard/nodes-overview-block.type';

@Component({
  selector: 'mina-nodes-live-filters',
  templateUrl: './nodes-live-filters.component.html',
  styleUrls: ['./nodes-live-filters.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-lg fx-row-vert-cent pl-12 border-bottom' },
})
export class NodesLiveFiltersComponent extends StoreDispatcher implements OnInit {

  readonly allFilters: string[] = [
    'best tip',
    NodesOverviewNodeBlockStatus.FETCHING,
    NodesOverviewNodeBlockStatus.FETCHED,
    NodesOverviewNodeBlockStatus.APPLYING,
    NodesOverviewNodeBlockStatus.APPLIED,
  ];
  activeFilters: string[] = [];

  ngOnInit(): void {
    this.listenToActiveFiltersChanges();
  }

  private listenToActiveFiltersChanges(): void {
    this.select(selectNodesLiveFilters, (filters: string[]) => {
      this.activeFilters = filters;
      this.detect();
    });
  }

  toggleFilter(filter: string): void {
    this.dispatch(NodesLiveToggleFilter, filter);
  }
}
