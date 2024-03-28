import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { tap, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import {
  NetworkNodeDhtClose,
  NetworkNodeDhtGetBootstrapStats,
  NetworkNodeDhtGetPeers,
  NetworkNodeDhtInit,
} from '@network/node-dht/network-node-dht.actions';
import { selectNetworkNodeDhtOpenSidePanel } from '@network/node-dht/network-node-dht.state';

@Component({
  selector: 'mina-network-node-dht',
  templateUrl: './network-node-dht.component.html',
  styleUrls: ['./network-node-dht.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkNodeDhtComponent extends StoreDispatcher implements OnInit, OnDestroy {

  openSidePanel: boolean;

  constructor(public el: ElementRef<HTMLElement>) { super(); }

  ngOnInit(): void {
    this.dispatch(NetworkNodeDhtInit);
    this.getPeers();
    this.listenToSidePanelChange();
  }

  private getPeers(): void {
    timer(3000, 3000)
      .pipe(
        tap(() => this.dispatch(NetworkNodeDhtGetPeers)),
        tap(() => this.dispatch(NetworkNodeDhtGetBootstrapStats)),
        untilDestroyed(this),
      )
      .subscribe();
  }

  private listenToSidePanelChange(): void {
    this.select(selectNetworkNodeDhtOpenSidePanel, open => {
      if (open && !this.openSidePanel) {
        this.openSidePanel = true;
        this.detect();
      } else if (!open && this.openSidePanel) {
        this.openSidePanel = false;
        this.detect();
      }
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(NetworkNodeDhtClose);
  }
}
