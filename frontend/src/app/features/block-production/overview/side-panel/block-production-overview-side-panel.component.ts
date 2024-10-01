import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionOverviewSelectors } from '@block-production/overview/block-production-overview.state';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { filter } from 'rxjs';
import { noMillisFormat, ONE_THOUSAND, toReadableDate } from '@openmina/shared';
import {
  BlockProductionOverviewAllStats,
} from '@shared/types/block-production/overview/block-production-overview-all-stats.type';
import { BlockProductionOverviewActions } from '@block-production/overview/block-production-overview.actions';
import {
  BlockProductionOverviewSlot,
} from '@shared/types/block-production/overview/block-production-overview-slot.type';
import { Routes } from '@shared/enums/routes.enum';
import { Router } from '@angular/router';
import {
  BlockProductionOverviewSlotsComponent,
} from '@block-production/overview/slots/block-production-overview-slots.component';

@Component({
  selector: 'mina-block-production-overview-side-panel',
  templateUrl: './block-production-overview-side-panel.component.html',
  styleUrls: ['./block-production-overview-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BlockProductionOverviewSidePanelComponent extends StoreDispatcher implements OnInit {

  activeScreen: number = 0;
  activeTab: number = 1;
  activeEpoch: BlockProductionOverviewEpoch;
  activeSlot: BlockProductionOverviewSlot;
  activeSlotStatus: string;
  activeSlotColor: string;

  extras: any = {
    balanceProducer: undefined,
    balanceDelegated: undefined,
    balanceStaked: undefined,
    epochStarted: undefined,
    epochEnds: undefined,
    slotsUsed: undefined,
  };
  singleEpochStats: BlockProductionOverviewAllStats = {
    totalSlots: undefined,
    wonSlots: undefined,
    canonical: undefined,
    orphaned: undefined,
    missed: undefined,
    futureRights: undefined,
    earnedRewards: undefined,
    expectedRewards: undefined,
  };
  allTimeStats: BlockProductionOverviewAllStats;

  constructor(private router: Router) {super();}

  ngOnInit(): void {
    this.listenToActiveEpoch();
    this.listenToAllTimeStats();
    this.listenToActiveSlot();
  }

  private listenToActiveEpoch(): void {
    this.select(BlockProductionOverviewSelectors.activeEpoch, (epoch: BlockProductionOverviewEpoch) => {
      this.activeEpoch = epoch;

      this.singleEpochStats = {
        totalSlots: epoch.details.totalSlots,
        wonSlots: epoch.details.wonSlots,
        canonical: epoch.details.canonical,
        orphaned: epoch.details.orphaned,
        missed: epoch.details.missed,
        futureRights: epoch.details.futureRights,
        earnedRewards: epoch.details.earnedRewards,
        expectedRewards: epoch.details.expectedRewards,
      };

      const startSlot = epoch.slots.find(slot => slot.globalSlot === epoch.details.slotStart);
      const endSlot = epoch.slots.find(slot => slot.globalSlot === epoch.details.slotEnd);
      this.extras = {
        balanceProducer: epoch.details.balanceProducer,
        balanceDelegated: epoch.details.balanceDelegated,
        balanceStaked: epoch.details.balanceStaked,
        epochStarted: startSlot ? toReadableDate(startSlot.time * ONE_THOUSAND, noMillisFormat) : '-',
        epochEnds: endSlot ? toReadableDate(endSlot.time * ONE_THOUSAND, noMillisFormat) : '-',
        slotsUsed: Math.round((epoch.details.canonical + epoch.details.orphaned + epoch.details.missed) / epoch.details.totalSlots * 100) + '%',
      };
      this.detect();
    }, filter(Boolean), filter(epoch => epoch.slots?.length > 0 && !!epoch.details));
  }

  private listenToAllTimeStats(): void {
    this.select(BlockProductionOverviewSelectors.allTimeStats, (stats: BlockProductionOverviewAllStats) => {
      this.allTimeStats = stats;
      this.detect();
    }, filter(Boolean));
  }

  private listenToActiveSlot(): void {
    this.select(BlockProductionOverviewSelectors.activeSlot, slot => {
      this.activeSlot = slot;
      if (slot) {
        this.activeScreen = 1;
        this.activeSlotStatus = BlockProductionOverviewSlotsComponent.getSlotText(this.activeSlot);
        this.activeSlotColor = BlockProductionOverviewSlotsComponent.getSlotColor(this.activeSlot, {
          canonical: true,
          orphaned: true,
          future: true,
          missed: true,
        });
      } else {
        this.activeScreen = 0;
      }
      this.detect();
    });
  }

  selectTab(tab: number): void {
    this.activeTab = tab;
  }

  removeActiveSlot(): void {
    this.dispatch2(BlockProductionOverviewActions.setActiveSlot({ slot: undefined }));
    this.router.navigate([Routes.BLOCK_PRODUCTION, Routes.OVERVIEW, this.activeEpoch.epochNumber], { queryParamsHandling: 'merge' });
  }
}
