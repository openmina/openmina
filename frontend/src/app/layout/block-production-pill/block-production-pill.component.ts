import { ChangeDetectionStrategy, Component, HostBinding, OnDestroy, OnInit } from '@angular/core';
import { AppSelectors } from '@app/app.state';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppNodeDetails } from '@shared/types/app/app-node-details.type';
import { getTimeDiff } from '@shared/helpers/date.helper';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { BlockProductionWonSlotsStatus } from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { filter } from 'rxjs';
import { isFeatureEnabled, isSubFeatureEnabled } from '@shared/constants/config';

@Component({
  selector: 'mina-block-production-pill',
  standalone: true,
  imports: [],
  templateUrl: './block-production-pill.component.html',
  styleUrl: './block-production-pill.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'mr-8 border-rad-6' },
})
export class BlockProductionPillComponent extends StoreDispatcher implements OnInit, OnDestroy {
  text: string = null;
  producingIn: string = null;

  private globalSlot: number = null;
  private interval: any;
  private producingValue: number = null;

  @HostBinding('class.hidden') private hideComponent = false;

  constructor(private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, (node) => {
      this.hideComponent = !isSubFeatureEnabled(node, 'block-production', 'won-slots');
    });
    this.select(AppSelectors.activeNodeDetails, (details: AppNodeDetails) => {
      if (details.producingBlockStatus === BlockProductionWonSlotsStatus.Committed) {
        this.text = 'active';
      } else if (details.producingBlockStatus === BlockProductionWonSlotsStatus.Produced) {
        this.text = 'done';
      } else {
        this.text = null;
      }
      this.globalSlot = details.producingBlockGlobalSlot;
      this.producingValue = details.producingBlockAt;
      this.producingIn = getTimeDiff(this.producingValue, { only1unit: true }).diff;
      this.detect();
    }, filter((details: AppNodeDetails) => !!details));
  }

  private clearInterval(): void {
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
    }
  }

  goToWonSlots(): void {
    if (!this.globalSlot) {
      return;
    }
    this.router.navigate([Routes.BLOCK_PRODUCTION, Routes.WON_SLOTS, this.globalSlot], { queryParamsHandling: 'merge' });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.clearInterval();
  }
}
