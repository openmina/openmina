import { ChangeDetectionStrategy, Component, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtClose, NetworkNodeDhtGetPeers } from '@network/node-dht/network-node-dht.actions';

@Component({
  selector: 'mina-dht-graph',
  templateUrl: './dht-graph.component.html',
  styleUrls: ['./dht-graph.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DhtGraphComponent extends StoreDispatcher implements OnInit, OnDestroy {

  ngOnInit(): void {
    // this.dispatch(NetworkNodeDhtGetPeers);
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    // this.dispatch(NetworkNodeDhtClose);
  }
}
