import { AfterViewInit, ChangeDetectionStrategy, Component, DestroyRef, ElementRef, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { SortDirection, TableSort } from '@openmina/shared';
import { HeartbeatSummary } from '@shared/types/leaderboard/heartbeat-summary.type';
import { LeaderboardSelectors } from '@leaderboard/leaderboard.state';
import { LeaderboardActions } from '@leaderboard/leaderboard.actions';
import { fromEvent } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

@Component({
  selector: 'mina-leaderboard-filters',
  templateUrl: './leaderboard-filters.component.html',
  styleUrl: './leaderboard-filters.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row flex-center w-100' },
})
export class LeaderboardFiltersComponent extends StoreDispatcher implements OnInit, AfterViewInit {

  protected readonly SortDirection = SortDirection;

  @ViewChild('inputElement') private inputElement: ElementRef<HTMLInputElement>;

  currentSort: TableSort<HeartbeatSummary>;

  constructor(private destroyRef: DestroyRef) {super();}

  ngOnInit(): void {
    this.listenToSort();
  }

  ngAfterViewInit(): void {
    fromEvent(this.inputElement.nativeElement, 'keyup')
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.dispatch2(LeaderboardActions.changeFilters({ filters: { search: this.inputElement.nativeElement.value } }));
      });
  }

  private listenToSort(): void {
    this.select(LeaderboardSelectors.sortBy, (sort: TableSort<HeartbeatSummary>) => {
      this.currentSort = sort;
      this.detect();
    });
  }

  sortBy(sortBy: string): void {
    const sortDirection = sortBy !== this.currentSort.sortBy
      ? this.currentSort.sortDirection
      : this.currentSort.sortDirection === SortDirection.ASC ? SortDirection.DSC : SortDirection.ASC;
    const sort = { sortBy: sortBy as keyof HeartbeatSummary, sortDirection };
    this.dispatch2(LeaderboardActions.sort({ sort }));
  }

}
