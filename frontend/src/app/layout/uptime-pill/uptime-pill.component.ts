import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { catchError, of, switchMap, timer } from 'rxjs';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { LeaderboardService } from '@leaderboard/leaderboard.service';
import { ManualDetection, OpenminaEagerSharedModule } from '@openmina/shared';

@UntilDestroy()
@Component({
  selector: 'mina-uptime-pill',
  standalone: true,
  imports: [
    OpenminaEagerSharedModule,
  ],
  templateUrl: './uptime-pill.component.html',
  styleUrl: './uptime-pill.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UptimePillComponent extends ManualDetection implements OnInit {

  uptime: { uptimePercentage: number, uptimeTime: string } = { uptimePercentage: 0, uptimeTime: '' };

  constructor(private leaderboardService: LeaderboardService) { super(); }

  ngOnInit(): void {
    this.listenToUptime();
  }

  private listenToUptime(): void {
    timer(0, 60000).pipe(
      switchMap(() => this.leaderboardService.getUptime()),
      catchError((err) => of({})),
      untilDestroyed(this),
    ).subscribe(uptime => {
      this.uptime = uptime;
      this.detect();
    });
  }
}
