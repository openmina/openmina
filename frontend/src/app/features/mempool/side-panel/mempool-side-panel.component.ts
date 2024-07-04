import { ChangeDetectionStrategy, Component, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { downloadJson, ExpandTracking, MinaJsonViewerComponent } from '@openmina/shared';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { MempoolActions } from '@app/features/mempool/mempool.actions';
import { MempoolSelectors } from '@app/features/mempool/mempool.state';
import { MempoolTransaction, SignedCommand, ZkappCommand } from '@shared/types/mempool/mempool-transaction.type';

@Component({
  selector: 'mina-mempool-side-panel',
  templateUrl: './mempool-side-panel.component.html',
  styleUrls: ['./mempool-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 flex-column' },
})
export class MempoolSidePanelComponent extends StoreDispatcher implements OnInit {

  tx: SignedCommand | ZkappCommand;
  jsonString: string;
  expandingTracking: ExpandTracking = {};

  @ViewChild(MinaJsonViewerComponent) private minaJsonViewer: MinaJsonViewerComponent;

  constructor(private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToActiveRowChange();
  }

  private listenToActiveRowChange(): void {
    this.select(MempoolSelectors.activeTx, (tx: MempoolTransaction) => {
      this.tx = tx?.transactionData;
      this.jsonString = JSON.stringify(tx?.transactionData);
      this.detect();
    });
  }

  closeSidePanel(): void {
    this.router.navigate([Routes.MEMPOOL], { queryParamsHandling: 'merge' });
    this.dispatch2(MempoolActions.setActiveTx({ tx: undefined }));
  }

  downloadJson(): void {
    downloadJson(this.jsonString, 'openmina-transaction.json');
  }

  expandEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(true);
  }

  collapseEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(false);
  }
}
