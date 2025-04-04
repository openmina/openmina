import { AfterViewInit, ChangeDetectionStrategy, Component, DestroyRef, ElementRef, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { LeaderboardActions } from '@leaderboard/leaderboard.actions';
import { debounceTime, fromEvent, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { trigger, state, style, animate, transition } from '@angular/animations';
import { ManualDetection } from '@openmina/shared';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { LeaderboardService } from '@leaderboard/leaderboard.service';

@Component({
  selector: 'mina-leaderboard-page',
  templateUrl: './leaderboard-page.component.html',
  styleUrl: './leaderboard-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
  animations: [
    trigger('expandCollapse', [
      state('false', style({
        height: '0',
        overflow: 'hidden',
        opacity: '0',
      })),
      state('true', style({
        height: '*',
        opacity: '1',
      })),
      transition('false <=> true', [
        animate('200ms ease-in-out'),
      ]),
    ]),
    trigger('rotateIcon', [
      state('false', style({ transform: 'rotate(0)' })),
      state('true', style({ transform: 'rotate(90deg)' })),
      transition('false <=> true', [
        animate('200ms'),
      ]),
    ]),
  ],
})
export class LeaderboardPageComponent extends StoreDispatcher implements OnInit, AfterViewInit {
  isExpanded = false;
  showBanner: boolean = false;
  canDownloadCSV = localStorage.getItem('download_leaderboard') === 'true';

  private readonly SCROLL_THRESHOLD = 100;
  @ViewChild('scrollContainer') private scrollContainer!: ElementRef;

  constructor(private destroyRef: DestroyRef,
              private leaderboardService: LeaderboardService) {
    super();
  }

  ngOnInit(): void {
    timer(0, 5000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch2(LeaderboardActions.getHeartbeats());
      });
  }

  ngAfterViewInit(): void {
    const container = this.scrollContainer.nativeElement;

    fromEvent(container, 'scroll')
      .pipe(
        debounceTime(100),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe(() => {
        const scrollPosition = container.scrollTop;

        if (scrollPosition > this.SCROLL_THRESHOLD && !this.showBanner) {
          this.showBanner = true;
          this.detect();
        } else if (scrollPosition <= this.SCROLL_THRESHOLD && this.showBanner) {
          this.showBanner = false;
          this.detect();
        }
      });
  }

  downloadUptimeLottery(): void {
    this.leaderboardService.downloadUptimeLottery();
  }

  downloadHighestUptime(): void {
    this.leaderboardService.downloadHighestUptime();
  }

  downloadMostProducedBlocks(): void {
    this.leaderboardService.downloadMostProducedBlocks();
  }

  downloadAll(): void {
    this.leaderboardService.downloadAll();
  }
}
