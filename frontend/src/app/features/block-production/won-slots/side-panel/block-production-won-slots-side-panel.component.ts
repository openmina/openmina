import { ChangeDetectionStrategy, Component, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionWonSlotsSelectors } from '@block-production/won-slots/block-production-won-slots.state';
import {
  BlockProductionWonSlotsSlot,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { getTimeDiff } from '@shared/helpers/date.helper';
import { hasValue, noMillisFormat, SecDurationConfig, toReadableDate } from '@openmina/shared';
import { filter } from 'rxjs';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';

@Component({
  selector: 'mina-block-production-won-slots-side-panel',
  templateUrl: './block-production-won-slots-side-panel.component.html',
  styleUrls: ['./block-production-won-slots-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BlockProductionWonSlotsSidePanelComponent extends StoreDispatcher implements OnInit, OnDestroy {

  readonly config: SecDurationConfig = {
    includeMinutes: true,
    color: false,
    onlySeconds: true,
    undefinedAlternative: undefined,
  };
  title: string;

  slot: BlockProductionWonSlotsSlot;
  percentage: number;
  remainingTime: string;
  scheduled: string;
  slotStartedAlready: boolean;

  vrfText: string;
  vrf: [number, number] = [0, 0];
  private timer: any;
  private stopTimer: boolean;
  private nextSlotStartingTime: number;

  ngOnInit(): void {
    this.listenToActiveSlot();
    this.parseRemainingTime();
  }

  private listenToActiveSlot(): void {
    this.select(BlockProductionWonSlotsSelectors.activeSlot, (slot: BlockProductionWonSlotsSlot) => {
      this.slot = slot;
      this.title = this.getTitle;

      this.scheduled = toReadableDate(slot.slotTime, noMillisFormat);
      this.slotStartedAlready = slot.slotTime < Date.now();

      this.nextSlotStartingTime = slot.slotTime + 180 * 1000;
      this.stopTimer = !this.slot.active;

      this.parse();

      const steps = [slot.times?.stagedLedgerDiffCreate, slot.times?.produced, slot.times?.proofCreate, slot.times?.blockApply];
      this.percentage = 25 * steps.filter(t => hasValue(t)).length;

      this.vrfText = this.getVrfText;
      this.vrf = slot.vrfValueWithThreshold;

      this.detect();
    }, filter(Boolean));
  }

  viewInMinaExplorer(): void {
    const url = `https://minaexplorer.com/block/${this.slot.hash}`;
    window.open(url, '_blank');
  }

  private get getTitle(): string {
    if (this.slot.active) {
      return 'Block production progress';
    }
    return 'Upcoming won slot';
  }

  private get getVrfText(): string {
    if (this.slot.active) {
      return 'Won Slot Requirement';
    }
    return 'Block Produce Right';
  }

  private parseRemainingTime(): void {
    this.timer = setInterval(() => {
      this.parse();
    }, 1000);
  }

  private parse(): void {
    if (this.slot && !this.stopTimer) {
      const remainingTime = getTimeDiff(this.nextSlotStartingTime - 270 * 1000, { withSecs: true });
      if (remainingTime.inFuture) {
        this.remainingTime = '-';
      }
      this.remainingTime = remainingTime.diff;
      if (this.remainingTime === '0s') {
        /* when we reached 0s, we need to fetch data again because this slot is over and the user should see that in the table */
        this.stopTimer = true;
        this.remainingTime = '-';
        setTimeout(() => {

          this.dispatch2(BlockProductionWonSlotsActions.getSlots());
        }, 1000);
      }
      this.detect();
    } else {
      this.remainingTime = '-';
    }
  }


  override ngOnDestroy(): void {
    super.ngOnDestroy();
    clearInterval(this.timer);
  }
}
