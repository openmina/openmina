import { ChangeDetectionStrategy, Component, OnDestroy, OnInit } from '@angular/core';
import { BenchmarksWalletsClose, BenchmarksWalletsGetWallets } from '@benchmarks/wallets/benchmarks-wallets.actions';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppSelectors } from '@app/app.state';
import { filter, skip } from 'rxjs';

@Component({
  selector: 'mina-benchmarks-wallets',
  templateUrl: './benchmarks-wallets.component.html',
  styleUrls: ['./benchmarks-wallets.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100' },
})
export class BenchmarksWalletsComponent extends StoreDispatcher implements OnInit, OnDestroy {

  ngOnInit(): void {
    this.listenToActiveNodeChange();
    this.dispatch(BenchmarksWalletsGetWallets, { initialRequest: true });
  }

  private listenToActiveNodeChange(): void {
    this.select(AppSelectors.activeNode, () => {
      this.dispatch(BenchmarksWalletsGetWallets, { initialRequest: true });
    }, filter(Boolean), skip(1));
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(BenchmarksWalletsClose);
  }
}
