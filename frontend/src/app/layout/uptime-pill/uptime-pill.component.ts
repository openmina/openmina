import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { catchError, mergeMap, of, switchMap, timer } from 'rxjs';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { LeaderboardService } from '@leaderboard/leaderboard.service';
import { ManualDetection, OpenminaEagerSharedModule } from '@openmina/shared';
import { sendSentryEvent } from '@shared/helpers/webnode.helper';

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
      mergeMap(() => this.leaderboardService.getUptime()),
      catchError(err => {
        sendSentryEvent(err.message);
        return of({
          uptimePercentage: 0,
          uptimeTime: '',
        });
      }),
      untilDestroyed(this),
    ).subscribe(uptime => {
      this.uptime = uptime;
      this.detect();
    });
  }
}
