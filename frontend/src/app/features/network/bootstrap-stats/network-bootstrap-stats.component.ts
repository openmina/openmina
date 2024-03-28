import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { tap, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import {
  NetworkBootstrapStatsClose,
  NetworkBootstrapStatsGetBootstrapStats,
  NetworkBootstrapStatsInit,
} from '@network/bootstrap-stats/network-bootstrap-stats.actions';
import {
  selectNetworkBootstrapStatsActiveBootstrapRequest,
} from '@network/bootstrap-stats/network-bootstrap-stats.state';

@Component({
  selector: 'mina-network-bootstrap-stats',
  templateUrl: './network-bootstrap-stats.component.html',
  styleUrls: ['./network-bootstrap-stats.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkBootstrapStatsComponent extends StoreDispatcher implements OnInit, OnDestroy {

  openSidePanel: boolean = false;

  constructor(public el: ElementRef<HTMLElement>) { super(); }

  ngOnInit(): void {
    this.dispatch(NetworkBootstrapStatsInit);
    this.getBootstrapStats();
    this.listenToSidePanelChange();
  }

  private getBootstrapStats(): void {
    timer(3000, 3000)
      .pipe(
        tap(() => this.dispatch(NetworkBootstrapStatsGetBootstrapStats)),
        untilDestroyed(this),
      )
      .subscribe();
  }

  private listenToSidePanelChange(): void {
    this.select(selectNetworkBootstrapStatsActiveBootstrapRequest, activeRow => {
      if (activeRow && !this.openSidePanel) {
        this.openSidePanel = true;
        this.detect();
      } else if (!activeRow && this.openSidePanel) {
        this.openSidePanel = false;
        this.detect();
      }
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(NetworkBootstrapStatsClose);
  }
}
