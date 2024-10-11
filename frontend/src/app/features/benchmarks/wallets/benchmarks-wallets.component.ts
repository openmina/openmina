import { ChangeDetectionStrategy, Component, OnDestroy, OnInit } from '@angular/core';
import { BenchmarksWalletsClose, BenchmarksWalletsGetWallets } from '@benchmarks/wallets/benchmarks-wallets.actions';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppSelectors } from '@app/app.state';
import { filter, skip } from 'rxjs';
import { BenchmarksWalletsZkService } from '@benchmarks/wallets/benchmarks-wallets-zk.service';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

@Component({
  selector: 'mina-benchmarks-wallets',
  templateUrl: './benchmarks-wallets.component.html',
  styleUrls: ['./benchmarks-wallets.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100' },
})
export class BenchmarksWalletsComponent extends StoreDispatcher implements OnInit, OnDestroy {

  isWebNode: boolean = false;

  constructor(private zkService: BenchmarksWalletsZkService) { super(); }

  ngOnInit(): void {
    this.zkService.loadO1js();
    this.listenToActiveNodeChange();
    this.dispatch(BenchmarksWalletsGetWallets, { initialRequest: true });
  }

  private listenToActiveNodeChange(): void {
    this.select(AppSelectors.activeNode, (node: MinaNode) => {
      this.isWebNode = node.isWebNode;
      this.detect();
    }, filter(Boolean));
    this.select(AppSelectors.activeNode, () => {
      this.dispatch(BenchmarksWalletsGetWallets, { initialRequest: true });
    }, filter(Boolean), skip(1));
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(BenchmarksWalletsClose);
  }
}
