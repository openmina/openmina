import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtState } from '@network/node-dht/network-node-dht.state';
import { selectNetworkNodeDhtState } from '@network/network.state';
import { filter, skip, take } from 'rxjs';

interface DhtPoint {
  left: number;
  distance: number;
  color: string;
  isBucket: boolean;
  isOrigin?: boolean;
  peerId?: string;
}

const randomColor = () => {
  return `#${Math.floor(Math.random() * 16777215).toString(16)}`;
};

const color50Array = Array.from({ length: 50 }, () => randomColor());

@Component({
  selector: 'mina-network-node-dht-line',
  templateUrl: './network-node-dht-line.component.html',
  styleUrls: ['./network-node-dht-line.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100 pl-12 pr-12' }
})
export class NetworkNodeDhtLineComponent extends StoreDispatcher implements OnInit {

  points: DhtPoint[] = [];
  buckets: DhtPoint[] = [];
  readonly trackPoints = (_: number, point: DhtPoint) => point.left;
  readonly trackBuckets = (_: number, bucket: DhtPoint) => bucket.left;

  ngOnInit(): void {
    this.listenToNodeDhtPeers();
  }

  private listenToNodeDhtPeers(): void {
    this.select(selectNetworkNodeDhtState, (state: NetworkNodeDhtState) => {
      this.points = [];
      this.buckets = [];
      const max_keyspace_hex = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';
      const max_keyspace_int = BigInt('0x' + max_keyspace_hex);
      const this_key_int = BigInt('0x' + state.thisKey);

      const buckets = Array.from(new Set(state.peers.map(peer => peer.bucketIndex)));
      for (const bucket of buckets.reverse()) {
        const bucket_peers = state.peers.filter(peer => peer.bucketIndex === bucket);
        const this_bucket_key_int = BigInt('0x' + bucket_peers[0].bucketMaxHex);
        const max_dist_int = Number(this_bucket_key_int) / Number(max_keyspace_int);
        const left_percent = (Number(this_bucket_key_int) / Number(max_keyspace_int)) * 100;
        this.buckets.push({
          left: left_percent,
          distance: max_dist_int,
          color: '',
          isBucket: true,
          peerId: ''
        });
      }

      for (const peer of state.peers) {
        const dist_int = BigInt('0x' + peer.hexDistance);
        const dist_normalized = (Number(dist_int) / Number(max_keyspace_int));
        this.points.push({
          left: dist_normalized * 100,
          distance: dist_normalized,
          color: '',
          isBucket: false,
          isOrigin: peer.key === state.thisKey,
          peerId: peer.peerId
        });
      }
      console.log(this.buckets);
      this.points.forEach((point, index) => point.color = color50Array[index]);
      this.detect();
    }, filter(state => !!state.thisKey));
  }
}
