import { ChangeDetectionStrategy, Component, OnInit, TemplateRef, ViewChild, ViewContainerRef } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  selectBlockProductionOverviewActiveEpoch,
  selectBlockProductionOverviewFilters,
  selectBlockProductionOverviewScale,
} from '@block-production/overview/block-production-overview.state';
import {
  BlockProductionOverviewFilters,
} from '@shared/types/block-production/overview/block-production-overview-filters.type';
import {
  BlockProductionOverviewChangeFilters,
  BlockProductionOverviewChangeScale,
  BlockProductionOverviewGetEpochDetails,
} from '@block-production/overview/block-production-overview.actions';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { filter } from 'rxjs';
import { TemplatePortal } from '@angular/cdk/portal';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';

@Component({
  selector: 'mina-block-production-overview-toolbar',
  templateUrl: './block-production-overview-toolbar.component.html',
  styleUrls: ['./block-production-overview-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'fx-row-vert-cent h-xl pl-12 f-600' },
})
export class BlockProductionOverviewToolbarComponent extends StoreDispatcher implements OnInit {

  filters: BlockProductionOverviewFilters;
  activeEpoch: BlockProductionOverviewEpoch;
  totalCanonical: number;
  totalOrphaned: number;
  totalMissed: number;
  totalFuture: number;
  scale: 'linear' | 'adaptive' = 'adaptive';

  @ViewChild('scaleDropdown') private dropdown: TemplateRef<void>;
  private overlayRef: OverlayRef;

  constructor(private overlay: Overlay,
              private viewContainerRef: ViewContainerRef) {super();}

  ngOnInit(): void {
    this.listenToScale();
    this.listenToFilters();
    this.listenToActiveEpoch();
  }

  private listenToFilters(): void {
    this.select(selectBlockProductionOverviewFilters, (filters: BlockProductionOverviewFilters) => {
      this.filters = filters;
      this.detect();
    });
  }

  private listenToActiveEpoch(): void {
    this.select(selectBlockProductionOverviewActiveEpoch, (activeEpoch: BlockProductionOverviewEpoch) => {
      this.activeEpoch = activeEpoch;
      if (activeEpoch?.slots) {
        this.totalCanonical = activeEpoch.slots.filter(s => s.canonical).length;
        this.totalOrphaned = activeEpoch.slots.filter(s => s.orphaned).length;
        this.totalMissed = activeEpoch.slots.filter(s => s.missed).length;
        this.totalFuture = activeEpoch.slots.filter(s => s.futureRights).length;
      } else {
        this.totalCanonical = 0;
        this.totalOrphaned = 0;
        this.totalMissed = 0;
        this.totalFuture = 0;
      }
      this.detect();
    }, filter(epoch => !!epoch?.slots));
  }

  private listenToScale(): void {
    this.select(selectBlockProductionOverviewScale, scale => {
      this.scale = scale;
      this.detect();
    });
  }

  changeFilter(filter: keyof BlockProductionOverviewFilters, value: boolean): void {
    this.dispatch(BlockProductionOverviewChangeFilters, { ...this.filters, [filter]: value });
  }

  changeActiveEpoch(newEpoch: number): void {
    this.dispatch(BlockProductionOverviewGetEpochDetails, newEpoch);
  }

  changeScale(scale: 'linear' | 'adaptive'): void {
    this.dispatch(BlockProductionOverviewChangeScale, scale);
    this.detach();
  }

  openDropdown(target: HTMLDivElement): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      return;
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      width: target.offsetWidth,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(target)
        .withPositions([{
          originX: 'start',
          originY: 'bottom',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 10,
        }]),
    });

    const portal = new TemplatePortal(this.dropdown, this.viewContainerRef);
    this.overlayRef.attach(portal);
  }

  detach(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }
}
