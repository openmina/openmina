import { AfterViewInit, ChangeDetectionStrategy, Component, DestroyRef, ElementRef, ViewChild } from '@angular/core';
import { ManualDetection } from '@openmina/shared';
import { debounceTime, fromEvent } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

@Component({
  selector: 'mina-leaderboard-landing-page',
  templateUrl: './leaderboard-landing-page.component.html',
  styleUrl: './leaderboard-landing-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 align-center' },
})
export class LeaderboardLandingPageComponent extends ManualDetection implements AfterViewInit {
  showBanner: boolean = false;

  private readonly SCROLL_THRESHOLD = 100;
  @ViewChild('scrollContainer') private scrollContainer!: ElementRef;

  constructor(private destroyRef: DestroyRef) {
    super();
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
}
