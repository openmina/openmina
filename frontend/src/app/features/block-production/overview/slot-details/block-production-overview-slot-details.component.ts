import { ChangeDetectionStrategy, Component, Input } from '@angular/core';
import {
  BlockProductionOverviewSlot,
} from '@shared/types/block-production/overview/block-production-overview-slot.type';
import { AppSelectors } from '@app/app.state';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppNodeDetails } from '@shared/types/app/app-node-details.type';
import { filter } from 'rxjs';
import { safelyExecuteInBrowser } from '@openmina/shared';

@Component({
  selector: 'mina-block-production-overview-slot-details',
  templateUrl: './block-production-overview-slot-details.component.html',
  styleUrls: ['./block-production-overview-slot-details.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-minus-xl flex-column' },
})
export class BlockProductionOverviewSlotDetailsComponent extends StoreDispatcher {
  @Input({ required: true }) activeSlot: BlockProductionOverviewSlot;

  private network: string;

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNodeDetails, (node: AppNodeDetails) => {
      this.network = node.network?.toLowerCase();
    }, filter(Boolean));
  }

  viewInMinaScan(): void {
    const url = `https://minascan.io/${this.network}/block/${this.activeSlot.hash}`;
    safelyExecuteInBrowser(() => window.open(url, '_blank'));
  }
}
