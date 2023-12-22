import { ChangeDetectionStrategy, Component, OnInit, ViewChild, ViewContainerRef } from '@angular/core';
import { selectNetworkBlocks } from '@network/blocks/network-blocks.state';
import { NetworkBlock } from '@shared/types/network/blocks/network-block.type';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BarGraphComponent } from '@openmina/shared';

@Component({
  selector: 'mina-network-blocks-graph',
  templateUrl: './network-blocks-graph.component.html',
  styleUrls: ['./network-blocks-graph.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkBlocksGraphComponent extends StoreDispatcher implements OnInit {

  bars: number[] = [];

  @ViewChild('minaBarGraph', { read: ViewContainerRef })
  private minaBarGraphRef: ViewContainerRef;
  private component: BarGraphComponent;

  async ngOnInit(): Promise<void> {
    await import('@openmina/shared').then((c) => {
      this.component = this.minaBarGraphRef.createComponent<BarGraphComponent>(c.BarGraphComponent).instance;
      this.component.xStep = 1;
      this.component.xTicksLength = 15;
      this.component.yTicksLength = 6;
      this.component.um = 's';
      this.component.yAxisLabel = 'Count';
      this.component.decimals = 0;
      this.component.responsive = false;
      this.component.ngOnInit();
    });
    this.listenToNetworkBlocks();
  }

  private listenToNetworkBlocks(): void {
    this.select(selectNetworkBlocks, (blocks: NetworkBlock[]) => {
      this.bars = blocks.filter(b => b.receivedLatency || b.sentLatency).map(b => b.receivedLatency || b.sentLatency);
      this.component.values = this.bars;
      this.component.update();
      this.component.detect();
      this.detect();
    });
  }
}

