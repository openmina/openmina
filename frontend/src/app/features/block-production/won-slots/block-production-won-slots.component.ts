import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { getMergedRoute, isDesktop, isMobile, MergedRoute } from '@openmina/shared';
import { debounceTime, filter, fromEvent, take, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';
import { AppSelectors } from '@app/app.state';

@Component({
  selector: 'mina-block-production-won-slots',
  templateUrl: './block-production-won-slots.component.html',
  styleUrls: ['./block-production-won-slots.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BlockProductionWonSlotsComponent extends StoreDispatcher implements OnInit, OnDestroy {

  showSidePanel: boolean = isDesktop();

  constructor(protected el: ElementRef) { super(); }

  ngOnInit(): void {
    this.listenToActiveNode();
    timer(10000, 10000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch2(BlockProductionWonSlotsActions.getSlots());
      });
    this.listenToResize();
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, () => {
      this.select(getMergedRoute, (data: MergedRoute) => {
        this.dispatch2(BlockProductionWonSlotsActions.init({ activeSlotRoute: data.params['id'] }));
      }, take(1));
    });
  }

  private listenToResize(): void {
    fromEvent(window, 'resize')
      .pipe(
        debounceTime(100),
        filter(() => this.showSidePanel === isMobile()),
        untilDestroyed(this),
      )
      .subscribe(() => {
        this.showSidePanel = isDesktop();
        this.detect();
      });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch2(BlockProductionWonSlotsActions.close());
  }
}
