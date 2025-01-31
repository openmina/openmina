import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { LeaderboardActions } from '@leaderboard/leaderboard.actions';
import { timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';

@Component({
  selector: 'mina-leaderboard-page',
  templateUrl: './leaderboard-page.component.html',
  styleUrl: './leaderboard-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class LeaderboardPageComponent extends StoreDispatcher implements OnInit {

  ngOnInit(): void {
    timer(0, 5000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch2(LeaderboardActions.getHeartbeats());
      });
  }

}
