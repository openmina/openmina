import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { LeaderboardSelectors } from '@leaderboard/leaderboard.state';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { isDesktop } from '@openmina/shared';

@Component({
  selector: 'mina-leaderboard-table',
  templateUrl: './leaderboard-table.component.html',
  styleUrl: './leaderboard-table.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100 h-100' },
})
export class LeaderboardTableComponent extends StoreDispatcher implements OnInit {

  isLoading: boolean;
  rows: HeartbeatSummary[] = [];
  desktop: boolean = isDesktop();

  ngOnInit(): void {
    this.listenToEmptyInDatabase();
    this.listenToHeartbeatsChanges();
  }

  private listenToEmptyInDatabase(): void {
    this.select(LeaderboardSelectors.isLoading, (loading: boolean) => {
      this.isLoading = loading;
      this.detect();
    });
  }

  private listenToHeartbeatsChanges(): void {
    this.select(LeaderboardSelectors.filteredHeartbeatSummaries, (rows: HeartbeatSummary[]) => {
      this.rows = rows;
      this.detect();
    });
  }
}
