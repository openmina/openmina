import { ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtState } from '@network/node-dht/network-node-dht.state';
import { selectNetworkNodeDhtState } from '@network/network.state';
import { filter } from 'rxjs';
import { NetworkNodeDhtToggleSidePanel } from '@network/node-dht/network-node-dht.actions';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';

interface DhtPoint {
  left: number;
  isBucket: boolean;
  distance?: number;
  isOrigin?: boolean;
  peerId?: string;
}

@Component({
  selector: 'mina-network-node-dht-line',
  templateUrl: './network-node-dht-line.component.html',
  styleUrls: ['./network-node-dht-line.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100 pl-12 pr-12' },
})
export class NetworkNodeDhtLineComponent extends StoreDispatcher implements OnInit {

  points: DhtPoint[] = [];
  buckets: DhtPoint[] = [];
  openSidePanel: boolean;

  readonly trackPoints = (_: number, point: DhtPoint) => point.left;
  readonly trackBuckets = (_: number, bucket: DhtPoint) => bucket.left;

  @ViewChild('line') private line: ElementRef<HTMLDivElement>;

  ngOnInit(): void {
    this.listenToNodeDhtPeers();
  }

  toggleSidePanel(): void {
    this.dispatch(NetworkNodeDhtToggleSidePanel);
  }

  private listenToNodeDhtPeers(): void {
    this.select(selectNetworkNodeDhtState, (state: NetworkNodeDhtState) => {
      this.calculateInitial(state);

      // this.calculate(state);
      console.log(this.points);
      this.openSidePanel = state.openSidePanel;
      this.detect();
    }, filter(state => !!state.thisKey));
  }

  private calculateInitial(state: NetworkNodeDhtState) {

    this.points = [];
    this.buckets = [];
    const max_keyspace_hex = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';
    const max_keyspace_int = BigInt('0x' + max_keyspace_hex);

    const buckets = Array.from(new Set(state.peers.map(peer => peer.bucketIndex)));
    for (const bucket of buckets.reverse()) {
      const bucket_peers = state.peers.filter(peer => peer.bucketIndex === bucket);
      const this_bucket_key_int = BigInt('0x' + bucket_peers[0].bucketMaxHex);
      const max_dist_int = Number(this_bucket_key_int) / Number(max_keyspace_int);
      const left_percent = (Number(this_bucket_key_int) / Number(max_keyspace_int)) * 100;
      this.buckets.push({
        left: left_percent,
        distance: max_dist_int,
        isBucket: true,
        peerId: '',
      });
    }

    for (const peer of state.peers) {
      const dist_int = BigInt('0x' + peer.hexDistance);
      const dist_normalized = (Number(dist_int) / Number(max_keyspace_int));
      this.points.push({
        left: dist_normalized * 100,
        distance: dist_normalized,
        isBucket: false,
        isOrigin: peer.key === state.thisKey,
        peerId: peer.peerId,
      });
    }

    const max = this.line.nativeElement.offsetWidth - 16;
    this.points = this.points.map(point => {
      return {
        ...point,
        left: (point.left / 100) * max,
      };
    });
    // this.buckets = this.buckets.map(bucket => {
    //   return {
    //     ...bucket,
    //     left: (bucket.left / 100) * max,
    //   };
    // });
  }

  private calculate(state: NetworkNodeDhtState) {
    this.points = [];
    this.buckets = [];
    const max_keyspace_hex = this.getMaxOfHex(state.buckets);
    const max_keyspace_int = BigInt('0x' + max_keyspace_hex);

    const buckets = state.buckets;
    for (const bucket of buckets.slice().reverse()) {
      const this_bucket_key_int = BigInt('0x' + bucket.bucketMaxHex);
      const left_percent = (Number(this_bucket_key_int) / Number(max_keyspace_int)) * 100;
      this.buckets.push({
        left: left_percent,
        isBucket: true,
        peerId: '',
      });
    }

    for (const peer of state.peers) {
      const dist_int = BigInt('0x' + peer.hexDistance);
      const dist_normalized = (Number(dist_int) / Number(max_keyspace_int));
      this.points.push({
        left: dist_normalized * 100,
        isBucket: false,
        isOrigin: peer.key === state.thisKey,
        peerId: peer.peerId,
      });
    }
  }

  private getMaxOfHex(buckets: NetworkNodeDhtBucket[]): string {
    return buckets.reduce((maxHex, bucket) => maxHex > bucket.bucketMaxHex ? maxHex : bucket.bucketMaxHex, '0');
  }
}
