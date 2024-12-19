import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionWonSlotsSelectors } from '@block-production/won-slots/block-production-won-slots.state';
import { lastItem, ONE_BILLION, ONE_THOUSAND } from '@openmina/shared';
import { getTimeDiff } from '@shared/helpers/date.helper';
import { filter } from 'rxjs';
import {
  BlockProductionWonSlotsSlot,
  BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import {
  BlockProductionWonSlotsEpoch,
} from '@shared/types/block-production/won-slots/block-production-won-slots-epoch.type';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';

@Component({
  selector: 'mina-block-production-won-slots-cards',
  templateUrl: './block-production-won-slots-cards.component.html',
  styleUrls: ['./block-production-won-slots-cards.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' },
})
export class BlockProductionWonSlotsCardsComponent extends StoreDispatcher implements OnInit {

  card1: { nextWonSlot: string; slot: number; } = { nextWonSlot: '-', slot: null };
  card2: { wonSlots: number; slotsUsed: number; } = { wonSlots: null, slotsUsed: null };
  card3: { acceptedBlocks: number; lastBlockTime: string; } = { acceptedBlocks: null, lastBlockTime: null };
  card4: { epochProgress: string; endIn: string; } = { epochProgress: '-', endIn: null };
  card5: { publicKey: string; totalRewards: string } = { publicKey: null, totalRewards: null };

  ngOnInit(): void {
    this.listenToSlots();
    this.listenToEpoch();
  }

  private listenToEpoch(): void {
    this.select(BlockProductionWonSlotsSelectors.epoch, (epoch: BlockProductionWonSlotsEpoch) => {
      const epochEndTime = this.addMinutesToTimestamp(epoch.currentTime / ONE_BILLION, (epoch.end - epoch.currentGlobalSlot) * 3);
      this.card4.endIn = getTimeDiff(epochEndTime * ONE_THOUSAND).diff;
      this.card4.epochProgress = Math.floor((epoch.currentGlobalSlot - epoch.start) / (epoch.end - epoch.start) * 100) + '%';
      this.card5.publicKey = epoch.publicKey;

      this.detect();
    }, filter(Boolean));
  }

  private listenToSlots(): void {
    this.select(BlockProductionWonSlotsSelectors.slots, (slots: BlockProductionWonSlotsSlot[]) => {
      const nextSlot = slots.find(s => s.status === BlockProductionWonSlotsStatus.Scheduled || !s.status);
      if (nextSlot) {
        this.card1.nextWonSlot = getTimeDiff(nextSlot.slotTime).diff;
        this.card1.slot = nextSlot.globalSlot;
      } else {
        this.card1.nextWonSlot = 'Now';
        this.card1.slot = slots.find(s => s.active)?.globalSlot;
      }

      this.card2.wonSlots = slots.length;
      this.card2.slotsUsed = slots.filter(
        s => [BlockProductionWonSlotsStatus.Canonical, BlockProductionWonSlotsStatus.Orphaned, BlockProductionWonSlotsStatus.Discarded].includes(s.status),
      ).length;

      this.card3.acceptedBlocks = slots.filter(s => s.status === BlockProductionWonSlotsStatus.Canonical).length;
      this.card3.lastBlockTime = getTimeDiff(lastItem(slots.filter(s => s.status === BlockProductionWonSlotsStatus.Canonical))?.slotTime).diff;

      this.card5.totalRewards = slots
        .filter(s => [BlockProductionWonSlotsStatus.Canonical].includes(s.status))
        .map(s => s.coinbaseRewards + s.txFeesRewards).reduce((a, b) => a + b, 0).toFixed(0);

      this.card5.totalRewards = isNaN(+this.card5.totalRewards) ? '0' : this.card5.totalRewards;
      this.detect();
    }, filter(slots => slots.length > 0));
  }

  private addMinutesToTimestamp(timestampInSeconds: number, minutesToAdd: number): number {
    const secondsToAdd = minutesToAdd * 60;
    return timestampInSeconds + secondsToAdd;
  }

  toggleSidePanel(): void {
    this.dispatch2(BlockProductionWonSlotsActions.toggleSidePanel());
  }
}
