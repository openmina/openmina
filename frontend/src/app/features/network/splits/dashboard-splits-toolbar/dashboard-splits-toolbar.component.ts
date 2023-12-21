import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { filter } from 'rxjs';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { DashboardSplitsSet } from '@shared/types/network/splits/dashboard-splits-set.type';
import { DashboardNodeCount } from '@shared/types/network/splits/dashboard-node-count.type';
import { selectLoadingStateLength } from '@app/layout/toolbar/loading.reducer';
import {
  selectDashboardSplitsNetworkMergeDetails,
  selectDashboardSplitsNetworkSplitsDetails,
  selectDashboardSplitsNodeStats, selectDashboardSplitsSets,
} from '@network/splits/dashboard-splits.state';
import { DashboardSplitsGetSplits, DashboardSplitsMergeNodes, DashboardSplitsSplitNodes } from '@network/splits/dashboard-splits.actions';

@Component({
  selector: 'mina-dashboard-splits-toolbar',
  templateUrl: './dashboard-splits-toolbar.component.html',
  styleUrls: ['./dashboard-splits-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-xl' },
})
export class DashboardSplitsToolbarComponent extends StoreDispatcher implements OnInit {

  setsLength: number;
  sets: DashboardSplitsSet[] = [];
  networkSplitTime: string;
  networkMergeTime: string;
  stats: DashboardNodeCount;
  fetching: boolean;

  ngOnInit(): void {
    this.listenToSetsChanges();
    this.listenToSplitTimeChanges();
    this.listenToMergeTimeChanges();
    this.listenToNodeStatsChanges();
  }

  private listenToSetsChanges(): void {
    this.select(selectLoadingStateLength, (length: number) => {
      this.fetching = length > 0;
      this.detect();
    }, filter((length: number) => this.fetching !== (length > 0)));

    this.select(selectDashboardSplitsSets, (sets: DashboardSplitsSet[]) => {
      this.setsLength = sets.length;
      this.sets = sets;
      this.detect();
    }, filter(sets => sets.length > 0));
  }

  private listenToSplitTimeChanges(): void {
    this.select(selectDashboardSplitsNetworkSplitsDetails, (time: string) => {
      this.networkSplitTime = time;
      this.detect();
    });
  }

  private listenToMergeTimeChanges(): void {
    this.select(selectDashboardSplitsNetworkMergeDetails, (time: string) => {
      this.networkMergeTime = time;
      this.detect();
    });
  }

  private listenToNodeStatsChanges(): void {
    this.select(selectDashboardSplitsNodeStats, (stats: DashboardNodeCount) => {
      this.stats = stats;
      this.detect();
    });
  }

  splitNodes(): void {
    this.dispatch(DashboardSplitsSplitNodes);
  }

  refresh(): void {
    this.dispatch(DashboardSplitsGetSplits);
  }

  mergeNodes(): void {
    this.dispatch(DashboardSplitsMergeNodes);
  }
}
