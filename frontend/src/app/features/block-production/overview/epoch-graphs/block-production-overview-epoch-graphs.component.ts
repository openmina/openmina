import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { BlockProductionOverviewSelectors } from '@block-production/overview/block-production-overview.state';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { filter } from 'rxjs';

@Component({
  selector: 'mina-block-production-overview-epoch-graphs',
  templateUrl: './block-production-overview-epoch-graphs.component.html',
  styleUrls: ['./block-production-overview-epoch-graphs.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row pl-12 mr-16' },
})
export class BlockProductionOverviewEpochGraphsComponent extends StoreDispatcher implements OnInit {

  epochs: BlockProductionOverviewEpoch[] = [];
  activeEpoch: BlockProductionOverviewEpoch;
  private max: number;
  private scale: 'linear' | 'adaptive' = 'adaptive';

  ngOnInit(): void {
    this.listenToScale();
    this.listenToEpochs();
  }

  private listenToScale(): void {
    this.select(BlockProductionOverviewSelectors.scale, (scale: 'linear' | 'adaptive') => {
      this.scale = scale;
      this.detect();
    });
  }

  private listenToEpochs(): void {
    this.select(BlockProductionOverviewSelectors.epochs, (epochs: BlockProductionOverviewEpoch[]) => {
      this.epochs = epochs;
      this.max = Math.max(...this.epochs.map(e => {
        return Math.max(
          ...e.windows.map(w => {
            return Math.max(w.canonical, w.orphaned, w.missed, w.futureRights);
          }),
        );
      }));
      this.detect();
    }, filter(epochs => epochs.length > 0));

    this.select(BlockProductionOverviewSelectors.activeEpoch, (activeEpoch: BlockProductionOverviewEpoch) => {
      this.activeEpoch = activeEpoch;
      this.detect();
    });
  }

  getHeight(data: number): number {
    if (this.scale === 'linear') {
      return data * 100 / 238;
    }

    const base = Math.ceil(Math.log10(this.max));
    const adjustedData = data + 1; // Add 1 to data to avoid log10(1) = 0
    return (Math.log10(adjustedData) / base) * 100;
  }
}
