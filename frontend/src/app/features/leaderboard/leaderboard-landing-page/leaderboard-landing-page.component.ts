import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';

@Component({
  selector: 'mina-leaderboard-landing-page',
  templateUrl: './leaderboard-landing-page.component.html',
  styleUrl: './leaderboard-landing-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 align-center' },
})
export class LeaderboardLandingPageComponent implements OnInit {

  ngOnInit(): void {
  }

}
