import { ChangeDetectionStrategy, Component, ElementRef, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionOverviewActions } from '@block-production/overview/block-production-overview.actions';
import { getMergedRoute, isDesktop, MergedRoute } from '@openmina/shared';
import { debounceTime, filter, fromEvent, take } from 'rxjs';
import { isNaN } from 'mathjs';
import { untilDestroyed } from '@ngneat/until-destroy';

@Component({
  selector: 'mina-block-production-overview',
  templateUrl: './block-production-overview.component.html',
  styleUrls: ['./block-production-overview.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BlockProductionOverviewComponent extends StoreDispatcher implements OnInit {

  showSidePanel: boolean = isDesktop();

  constructor(protected el: ElementRef) { super(); }

  ngOnInit(): void {
    this.dispatch2(BlockProductionOverviewActions.getRewardsStats());
    this.listenToRoute();
    this.listenToResize();
  }

  private listenToRoute(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      const epoch = Number(route.params['epoch']);
      this.dispatch2(BlockProductionOverviewActions.getEpochDetails({ epochNumber: isNaN(epoch) ? undefined : epoch }));
    }, take(1));
  }

  private listenToResize(): void {
    fromEvent(window, 'resize')
      .pipe(
        debounceTime(100),
        filter(() => this.showSidePanel !== isDesktop()),
        untilDestroyed(this),
      )
      .subscribe(() => {
        this.showSidePanel = isDesktop();
        this.detect();
      });
  }
}
