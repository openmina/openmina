import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionWonSlotsSelectors } from '@block-production/won-slots/block-production-won-slots.state';
import { filter } from 'rxjs';
import { BlockProductionWonSlotsSlot } from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { BlockProductionWonSlotsEpoch } from '@shared/types/block-production/won-slots/block-production-won-slots-epoch.type';
import { ONE_BILLION, ONE_THOUSAND } from '@openmina/shared';
import { getTimeDiff } from '@shared/helpers/date.helper';

@Component({
  selector: 'mina-block-production-won-slots-epoch',
  templateUrl: './block-production-won-slots-epoch.component.html',
  styleUrl: './block-production-won-slots-epoch.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-lg pl-12 f-600 fx-row-vert-cent border-bottom' },
})
export class BlockProductionWonSlotsEpochComponent extends StoreDispatcher implements OnInit {
  epoch: string;
  startedAgo: string;

  ngOnInit(): void {
    this.listenToEpoch();
    this.listenToEpoch2();
  }

  private listenToEpoch(): void {
    this.select(BlockProductionWonSlotsSelectors.slots, (slots: BlockProductionWonSlotsSlot[]) => {
      this.epoch = 'Epoch ' + slots[0].epoch;
      this.detect();
    }, filter(slots => slots.length > 0));
  }

  private listenToEpoch2(): void {
    this.select(BlockProductionWonSlotsSelectors.epoch, (epoch: BlockProductionWonSlotsEpoch) => {
      const epochStartTime = this.addMinutesToTimestamp(Math.floor(epoch.currentTime / ONE_BILLION), -(epoch.currentGlobalSlot - epoch.start) * 3);
      this.startedAgo = getTimeDiff(Math.floor(epochStartTime * ONE_THOUSAND)).diff;

      this.detect();
    }, filter(Boolean));
  }

  private addMinutesToTimestamp(timestampInSeconds: number, minutesToAdd: number): number {
    const secondsToAdd = minutesToAdd * 60;
    return timestampInSeconds + secondsToAdd;
  }
}
