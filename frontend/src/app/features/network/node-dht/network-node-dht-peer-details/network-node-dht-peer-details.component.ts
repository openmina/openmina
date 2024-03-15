import { ChangeDetectionStrategy, Component, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';
import { selectNetworkNodeDhtActivePeer } from '@network/node-dht/network-node-dht.state';
import { downloadJson, ExpandTracking, MinaJsonViewerComponent } from '@openmina/shared';
import { delay, mergeMap, of } from 'rxjs';

@Component({
  selector: 'mina-network-node-dht-peer-details',
  templateUrl: './network-node-dht-peer-details.component.html',
  styleUrls: ['./network-node-dht-peer-details.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkNodeDhtPeerDetailsComponent extends StoreDispatcher implements OnInit {

  activePeer: NetworkNodeDHT;
  expandingTracking: ExpandTracking = {};
  jsonString: string;

  @ViewChild(MinaJsonViewerComponent) private minaJsonViewer: MinaJsonViewerComponent;

  ngOnInit(): void {
    this.listenToActiveNode();
  }

  private listenToActiveNode(): void {
    this.select(selectNetworkNodeDhtActivePeer, (peer: NetworkNodeDHT) => {
      this.activePeer = peer;
      this.jsonString = JSON.stringify(peer);
      this.detect();
    }, mergeMap((peer: NetworkNodeDHT) => of(peer).pipe(delay(peer ? 0 : 300))));
  }

  downloadJson(): void {
    downloadJson(this.jsonString, 'dht-peer.json');
  }

  expandEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(true);
  }

  collapseEntireJSON(): void {
    this.expandingTracking = this.minaJsonViewer.toggleAll(false);
  }
}
