import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionOverviewActions } from '@block-production/overview/block-production-overview.actions';
import { getMergedRoute, isDesktop, MergedRoute } from '@openmina/shared';
import { debounceTime, filter, fromEvent, skip, take } from 'rxjs';
import { isNaN } from 'mathjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { BlockProductionOverviewSelectors } from '@block-production/overview/block-production-overview.state';
import { SLOTS_PER_EPOCH } from '@shared/constants/mina';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { AppSelectors } from '@app/app.state';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

@Component({
  selector: 'mina-block-production-overview',
  templateUrl: './block-production-overview.component.html',
  styleUrls: ['./block-production-overview.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BlockProductionOverviewComponent extends StoreDispatcher implements OnInit, OnDestroy {

  readonly SLOTS_PER_EPOCH = SLOTS_PER_EPOCH;
  showSidePanel: boolean = isDesktop();
  isLoading: boolean = true;
  isCalculatingVRF: boolean = false;
  epoch: BlockProductionOverviewEpoch;

  constructor(protected el: ElementRef) { super(); }

  ngOnInit(): void {
    this.listenToLoading();
    this.listenToResize();
    this.listenToNodeChange();
  }

  private listenToNodeChange(): void {
    this.select(AppSelectors.activeNode, () => {
      this.dispatch2(BlockProductionOverviewActions.getRewardsStats());
      this.listenToRoute();
    });
  }

  private listenToLoading(): void {
    this.select(BlockProductionOverviewSelectors.loading, ({ isLoading, isCalculatingVRF }) => {
      this.isLoading = isLoading;
      this.isCalculatingVRF = isCalculatingVRF;
      this.detect();
    });
    this.select(BlockProductionOverviewSelectors.activeEpoch, (activeEpoch: BlockProductionOverviewEpoch) => {
      this.epoch = activeEpoch;
      this.detect();
    });
  }

  private listenToRoute(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      const epoch = Number(route.params['epoch']);
      const slot = Number(route.params['slot']);
      this.dispatch2(BlockProductionOverviewActions.setActiveSlot({ slot }));
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

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch2(BlockProductionOverviewActions.close());
  }
}
