import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { LeaderboardActions } from '@leaderboard/leaderboard.actions';

@Component({
  selector: 'mina-leaderboard-landing-page',
  templateUrl: './leaderboard-landing-page.component.html',
  styleUrl: './leaderboard-landing-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 align-center' },
})
export class LeaderboardLandingPageComponent extends StoreDispatcher implements OnInit {

  ngOnInit(): void {
    this.dispatch2(LeaderboardActions.getHeartbeats());
  }

}
