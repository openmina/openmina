import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { selectNetworkBlocks } from '@network/blocks/network-blocks.state';
import { NetworkBlock } from '@shared/types/network/blocks/network-block.type';
import { SecDurationConfig } from '@openmina/shared';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-network-blocks-side-panel',
  templateUrl: './network-blocks-side-panel.component.html',
  styleUrls: ['./network-blocks-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 flex-column border-left' },
})
export class NetworkBlocksSidePanelComponent extends StoreDispatcher implements OnInit {

  readonly secConfig: SecDurationConfig = { onlySeconds: true, undefinedAlternative: '-', color: true, severe: 30, warn: 5 };

  firstSentTime: number;

  ngOnInit(): void {
    this.listenToBlocks();
  }

  private listenToBlocks(): void {
    this.select(selectNetworkBlocks, (blocks: NetworkBlock[]) => {
      const values = blocks.map(b => b.sentLatency).filter(Boolean);
      this.firstSentTime = values.length ? Math.min(...values) : undefined;
      this.detect();
    });
  }
}
