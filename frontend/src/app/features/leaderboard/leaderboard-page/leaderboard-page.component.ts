import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { LeaderboardActions } from '@leaderboard/leaderboard.actions';

@Component({
  selector: 'mina-leaderboard-page',
  templateUrl: './leaderboard-page.component.html',
  styleUrl: './leaderboard-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class LeaderboardPageComponent extends StoreDispatcher implements OnInit {

  ngOnInit(): void {
    this.dispatch2(LeaderboardActions.getHeartbeats());
  }

}
