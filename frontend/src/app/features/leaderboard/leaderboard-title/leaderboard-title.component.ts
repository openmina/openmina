import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { LeaderboardSelectors } from '@leaderboard/leaderboard.state';

@Component({
  selector: 'mina-leaderboard-title',
  templateUrl: './leaderboard-title.component.html',
  styleUrl: './leaderboard-title.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' },
})
export class LeaderboardTitleComponent extends StoreDispatcher implements OnInit {

  rows: HeartbeatSummary[] = [];

  ngOnInit(): void {
    this.listenToHeartbeatsChanges();
  }

  private listenToHeartbeatsChanges(): void {
    this.select(LeaderboardSelectors.filteredHeartbeatSummaries, (rows: HeartbeatSummary[]) => {
      this.rows = rows;
      this.detect();
    });
  }
}
