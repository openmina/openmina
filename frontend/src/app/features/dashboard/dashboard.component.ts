import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { DashboardGetData, DashboardInit } from '@dashboard/dashboard.actions';
import { tap, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';

@Component({
  selector: 'mina-dashboard',
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 w-100 flex-row' },
})
export class DashboardComponent extends StoreDispatcher implements OnInit {

  ngOnInit(): void {
    this.dispatch(DashboardInit);
    timer(2000, 2000)
      .pipe(
        tap(() => this.dispatch(DashboardGetData)),
        untilDestroyed(this),
      )
      .subscribe();
  }
}
