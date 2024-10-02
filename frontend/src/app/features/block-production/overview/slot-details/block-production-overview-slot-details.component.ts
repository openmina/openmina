import { ChangeDetectionStrategy, Component, Input } from '@angular/core';
import {
  BlockProductionOverviewSlot,
} from '@shared/types/block-production/overview/block-production-overview-slot.type';
import { AppSelectors } from '@app/app.state';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppNodeDetails } from '@shared/types/app/app-node-details.type';
import { filter } from 'rxjs';

@Component({
  selector: 'mina-block-production-overview-slot-details',
  templateUrl: './block-production-overview-slot-details.component.html',
  styleUrls: ['./block-production-overview-slot-details.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-minus-xl flex-column' },
})
export class BlockProductionOverviewSlotDetailsComponent extends StoreDispatcher {
  @Input({ required: true }) activeSlot: BlockProductionOverviewSlot;

  private minaExplorer: string;

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNodeDetails, (node: AppNodeDetails) => {
      this.minaExplorer = node.network?.toLowerCase();
    }, filter(Boolean));
  }

  viewInMinaExplorer(): void {
    const network = this.minaExplorer !== 'mainnet' ? (this.minaExplorer + '.') : '';
    const url = `https://${network}minaexplorer.com/block/${this.activeSlot.hash}`;
    window.open(url, '_blank');
  }
}
