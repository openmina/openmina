import { ChangeDetectionStrategy, Component, Input, OnChanges } from '@angular/core';
import {
  BlockProductionOverviewSlot,
} from '@shared/types/block-production/overview/block-production-overview-slot.type';
import { AppSelectors } from '@app/app.state';
import { CONFIG } from '@shared/constants/config';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-block-production-overview-slot-details',
  templateUrl: './block-production-overview-slot-details.component.html',
  styleUrls: ['./block-production-overview-slot-details.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-minus-xl flex-column' },
})
export class BlockProductionOverviewSlotDetailsComponent extends StoreDispatcher implements OnChanges {
  @Input({ required: true }) activeSlot: BlockProductionOverviewSlot;

  private minaExplorer: string;

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  ngOnChanges(): void {
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, (node) => {
      this.minaExplorer = node.minaExplorerNetwork ?? CONFIG.globalConfig?.minaExplorerNetwork;
    });
  }

  viewInMinaExplorer(): void {
    const network = this.minaExplorer !== 'mainnet' ? this.minaExplorer : '';
    const url = `https://${network}.minaexplorer.com/block/${this.activeSlot.hash}`;
    window.open(url, '_blank');
  }
}
