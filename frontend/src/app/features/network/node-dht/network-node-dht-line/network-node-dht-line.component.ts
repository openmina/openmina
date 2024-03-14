import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtState, selectNetworkNodeDhtPeers } from '@network/node-dht/network-node-dht.state';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';
import { selectNetworkNodeDhtState } from '@network/network.state';
import { isNaN } from 'mathjs';
import { take } from 'rxjs';
// export interface NetworkNodeDHT {
//   peerId: string;
//   key: string;
//   addressesLength: number;
//   addrs: string[];
//   hexDistance: string;
//   binaryDistance: string;
//   xorDistance: string;
//   bucketIndex: number;
// }

interface DhtPoint {
  left: number;
  distance: number;
  color: string;
  isBucket: boolean;
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

  ngOnInit(): void {
    this.listenToNodeDhtPeers();
  }

  private listenToNodeDhtPeers(): void {
    this.select(selectNetworkNodeDhtState, (state: NetworkNodeDhtState) => {
      /*
      *
# Load the JSON data
with open('/mnt/data/routing_table_3.json', 'r') as file:
    data = json.load(file)

# Maximum possible distance in hexadecimal
max_keyspace_hex = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'
max_keyspace_int = int(max_keyspace_hex, 16)

# Convert the node's own key to integer for reference
this_key_int = int(data['this_key'], 16)

# Initialize lists to hold max_dist and entry distances
max_dists = []
entry_dists = []

# Parse through the buckets to extract max_dist and entry distances
for bucket in data['buckets']:
    # Convert max_dist to integer and normalize it to the keyspace range
    max_dist_int = int(bucket['max_dist'], 16) / max_keyspace_int
    max_dists.append(max_dist_int)

    # Iterate through entries to extract and normalize distances
    for entry in bucket['entries']:
        dist_int = int(entry['dist'], 16) / max_keyspace_int
        entry_dists.append(dist_int)

# Sort the lists for better visualization
max_dists = sorted(max_dists)
entry_dists = sorted(entry_dists)
      * */

      // left is basically
      // this.points = [];
      // const max_keyspace_hex = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';
      // const max_keyspace_int = parseInt(max_keyspace_hex, 16);
      // const this_key_int = parseInt(state.thisKey, 16);
      // const max_dists = [];
      // const entry_dists = [];
      // for (const peer of state.peers) {
      //   const max_dist_int = parseInt(peer.bucketMaxHex, 16) / max_keyspace_int;
      //   this.points.push({ left: max_dist_int, color: randomColor(), isBucket: true });
      //   const dist_int = parseInt(peer.xorDistance, 16) / max_keyspace_int;
      //   this.points.push({ left: dist_int, color: randomColor(), isBucket: false });
      // }
      // console.log(this.points);
      // this.detect();

      this.points = [];
      const max_keyspace_hex = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';
      const max_keyspace_int = BigInt('0x' + max_keyspace_hex);
      const this_key_int = BigInt('0x' + state.thisKey);

      const this_key_proportion = Number(this_key_int) / Number(max_keyspace_int);

      const buckets = Array.from(new Set(state.peers.map(peer => peer.bucketIndex)));
      for (const bucket of buckets) {
        const bucket_peers = state.peers.filter(peer => peer.bucketIndex === bucket);
        const this_bucket_key_int = BigInt('0x' + bucket_peers[0].bucketMaxHex);
        const max_dist_int = Number(this_bucket_key_int) / Number(max_keyspace_int);
        // Calculate left as a percentage of 100%
        const left_percent = (Number(this_bucket_key_int) / Number(max_keyspace_int)) * 100;
        this.points.push({
          left: left_percent,
          distance: max_dist_int * 100, // Normalize the distance
          color: '',
          isBucket: true,
          peerId: ''
        });
      }

      for (const peer of state.peers.filter(peer => peer.key !== state.thisKey)) {
        const dist_int = BigInt('0x' + peer.hexDistance);
        const dist_normalized = (Number(dist_int) / Number(max_keyspace_int)) * 100;
        this.points.push({
          left: dist_normalized,
          distance: dist_normalized,
          color: '',
          isBucket: false,
          peerId: peer.peerId
        });
      }
      this.points.forEach((point, index) => point.color = color50Array[index]);
      console.log(this.points);
      this.detect();
    });
  }
}
