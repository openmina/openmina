import { ChangeDetectionStrategy, Component, ElementRef, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { tap, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { NetworkNodeDhtGetPeers, NetworkNodeDhtInit } from '@network/node-dht/network-node-dht.actions';

@Component({
  selector: 'mina-network-node-dht',
  templateUrl: './network-node-dht.component.html',
  styleUrls: ['./network-node-dht.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class NetworkNodeDhtComponent extends StoreDispatcher implements OnInit {

  constructor(public el: ElementRef<HTMLElement>) { super(); }

  ngOnInit(): void {
    this.dispatch(NetworkNodeDhtInit);
    timer(3000, 3000)
      .pipe(
        tap(() => this.dispatch(NetworkNodeDhtGetPeers)),
        untilDestroyed(this),
      )
      .subscribe();
  }
}
