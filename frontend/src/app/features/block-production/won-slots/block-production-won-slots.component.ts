import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { getMergedRoute, isDesktop, MergedRoute } from '@openmina/shared';
import { take, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';
import { AppSelectors } from '@app/app.state';
import { BlockProductionWonSlotsSelectors } from '@block-production/won-slots/block-production-won-slots.state';
import { AppNodeDetails, AppNodeStatus } from '@shared/types/app/app-node-details.type';
import { animate, style, transition, trigger } from '@angular/animations';

@Component({
  selector: 'mina-block-production-won-slots',
  templateUrl: './block-production-won-slots.component.html',
  styleUrls: ['./block-production-won-slots.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  animations: [
    trigger('fadeInOut', [
      transition(':enter', [
        style({ opacity: 0 }),
        animate('400ms ease-in', style({ opacity: 1 })),
      ]),
      transition(':leave', [
        animate('400ms ease-out', style({ opacity: 0 })),
      ]),
    ]),
  ],
})
export class BlockProductionWonSlotsComponent extends StoreDispatcher implements OnInit, OnDestroy {

  showSidePanel: boolean = isDesktop();
  isDesktop: boolean = isDesktop();
  nodeIsBootstrapping: boolean = false;
  isPending: boolean = true;
  isCalculatingVRF: boolean = false;
  vrfStats: {
    evaluated: number;
    total: number;
  };
  epoch: number;
  emptySlots: boolean = true;
  isLoading: boolean = true;

  constructor(protected el: ElementRef) { super(); }

  ngOnInit(): void {
    this.listenToActiveNode();
    this.listenToNodeChange();
    timer(10000, 10000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch2(BlockProductionWonSlotsActions.getSlots());
      });
    this.listenToResize();
    this.listenToActiveEpoch();
    this.listenToSlots();
  }

  private listenToNodeChange(): void {
    this.select(AppSelectors.activeNodeDetails, (node: AppNodeDetails) => {
      this.nodeIsBootstrapping = node?.status === AppNodeStatus.BOOTSTRAP;
      this.isPending = node?.status === AppNodeStatus.PENDING;
      this.detect();
    });
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, () => {
      this.select(getMergedRoute, (data: MergedRoute) => {
        this.isLoading = true;
        this.dispatch2(BlockProductionWonSlotsActions.init({ activeSlotRoute: data.params['id'] }));
      }, take(1));
    });
  }

  private listenToResize(): void {
    this.select(BlockProductionWonSlotsSelectors.openSidePanel, (open: boolean) => {
      this.showSidePanel = open;
      this.detect();
    });
  }

  private listenToActiveEpoch(): void {
    this.select(BlockProductionWonSlotsSelectors.epoch, (activeEpoch) => {
      this.epoch = activeEpoch?.epochNumber;
      this.vrfStats = activeEpoch.vrfStats;
      this.isCalculatingVRF = activeEpoch.vrfStats?.evaluated < activeEpoch.vrfStats?.total;
      this.detect();
    });
  }

  private listenToSlots(): void {
    this.select(BlockProductionWonSlotsSelectors.slots, (slots) => {
      const emptySlots = slots.length === 0;
      if (emptySlots !== this.emptySlots) {
        this.emptySlots = emptySlots;
        this.detect();
      }
    });
    this.select(BlockProductionWonSlotsSelectors.serverResponded, (responded) => {
      this.isLoading = !responded;
      this.detect();
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch2(BlockProductionWonSlotsActions.close());
  }
}
