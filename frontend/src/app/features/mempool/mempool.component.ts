import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { AppSelectors } from '@app/app.state';
import { MempoolSelectors } from '@app/features/mempool/mempool.state';
import { MempoolActions } from '@app/features/mempool/mempool.actions';

@Component({
  selector: 'mina-mempool',
  templateUrl: './mempool.component.html',
  styleUrls: ['./mempool.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class MempoolComponent extends StoreDispatcher implements OnInit, OnDestroy {

  isActiveRow: boolean;

  constructor(protected el: ElementRef) { super(); }

  ngOnInit(): void {
    this.listenToActiveNode();
    timer(10000, 3000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch2(MempoolActions.getTxs());
      });
    this.listenToSidePanelChange();
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, () => {
      this.dispatch2(MempoolActions.init());
    });
  }

  private listenToSidePanelChange(): void {
    this.select(MempoolSelectors.activeTx, activeTx => {
      if (activeTx && !this.isActiveRow) {
        this.isActiveRow = true;
        this.detect();
      } else if (!activeTx && this.isActiveRow) {
        this.isActiveRow = false;
        this.detect();
      }
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch2(MempoolActions.close());
  }
}
