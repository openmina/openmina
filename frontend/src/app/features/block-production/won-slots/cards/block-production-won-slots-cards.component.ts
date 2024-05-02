import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionWonSlotsSelectors } from '@block-production/won-slots/block-production-won-slots.state';
import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import { lastItem, ONE_THOUSAND } from '@openmina/shared';
import { getTimeDiff } from '@shared/helpers/date.helper';
import { filter } from 'rxjs';
import {
  BlockProductionWonSlotsSlot,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';

@Component({
  selector: 'mina-block-production-won-slots-cards',
  templateUrl: './block-production-won-slots-cards.component.html',
  styleUrls: ['./block-production-won-slots-cards.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' },
})
export class BlockProductionWonSlotsCardsComponent extends StoreDispatcher implements OnInit {

  epoch: BlockProductionOverviewEpochDetails;
  card1: { nextWonSlot: string; slot: number; } = { nextWonSlot: '-', slot: null };
  card2: { wonSlots: number; slotsUsed: number; } = { wonSlots: null, slotsUsed: null };
  card3: { acceptedBlocks: number; lastBlockTime: string; } = { acceptedBlocks: null, lastBlockTime: null };
  card4: { epochProgress: string; endIn: string; } = { epochProgress: '-', endIn: null };
  card5: { totalRewards: string; } = { totalRewards: '-' };

  ngOnInit(): void {
    this.listenToActiveEpoch();
    this.listenToSlots();
  }

  private listenToActiveEpoch(): void {
    this.select(BlockProductionWonSlotsSelectors.epoch, epoch => {
      this.epoch = epoch;
      this.card5.totalRewards = epoch?.earnedRewards;
      this.detect();
    });
  }

  private listenToSlots(): void {
    this.select(BlockProductionWonSlotsSelectors.slots, (slots: BlockProductionWonSlotsSlot[]) => {
      const nextSlot = slots.find(s => !s.status);
      if (nextSlot) {
        this.card1.nextWonSlot = getTimeDiff(nextSlot.slotTime).diff;
        this.card1.slot = nextSlot.globalSlot;
      } else {
        this.card1.nextWonSlot = 'Now';
        this.card1.slot = slots.find(s => s.active)?.globalSlot;
      }

      this.card2.wonSlots = slots.length;
      this.card2.slotsUsed = slots.filter(s => s.status).length;

      this.card3.acceptedBlocks = 0; //slots.filter(s => s.canonical).length;
      this.card3.lastBlockTime = null;//getTimeDiff(lastItem(slots.filter(s => s.canonical)).time).diff;

      const someSlot = slots[0];
      const currentTimeInSecs = Math.floor(new Date().getTime() / ONE_THOUSAND);
      const someSlotDiffUntilEnd = this.epoch.slotEnd - someSlot.globalSlot;
      const epochEndTimeInSecs = this.addMinutesToTimestamp(someSlot.slotTime, someSlotDiffUntilEnd * 3);

      this.card4.epochProgress = Math.floor((currentTimeInSecs - someSlot.slotTime) / (epochEndTimeInSecs - someSlot.slotTime) * 100) + '%';
      this.card4.endIn = getTimeDiff(epochEndTimeInSecs).diff;

      this.detect();
    }, filter(slots => slots.length > 0));
  }

  private addMinutesToTimestamp(timestampInSeconds: number, minutesToAdd: number): number {
    const secondsToAdd = minutesToAdd * 60;
    return timestampInSeconds + secondsToAdd;
  }
}
