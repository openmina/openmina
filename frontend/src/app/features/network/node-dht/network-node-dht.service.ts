import { Injectable } from '@angular/core';
import { RustService } from '@core/services/rust.service';
import { map, Observable } from 'rxjs';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBootstrapStats } from '@shared/types/network/node-dht/network-node-dht-bootstrap-stats.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';

@Injectable({
  providedIn: 'root',
})
export class NetworkNodeDhtService {

  constructor(private rust: RustService) { }

  getDhtPeers(): Observable<{ peers: NetworkNodeDhtPeer[], thisKey: string, buckets: NetworkNodeDhtBucket[] }> {
    return this.rust.get<DhtPeersResponse>('/discovery/routing_table').pipe(
      map((response: DhtPeersResponse) => this.mapDhtPeers(response)),
    );
  }

  getDhtBootstrapStats(): Observable<NetworkNodeDhtBootstrapStats[]> {
    return this.rust.get('/discovery/bootstrap_stats').pipe(
      map((response: any) => this.mapBootstrapStats(response)),
    );
  }

  private mapDhtPeers(response: DhtPeersResponse): {
    peers: NetworkNodeDhtPeer[],
    thisKey: string,
    buckets: NetworkNodeDhtBucket[]
  } {
    return {
      peers: response.buckets.reduce((acc, bucket) => {
        const nodes = bucket.entries.map(entry => {
          const binaryDistance = this.hexToBinary(entry.dist);
          return {
            peerId: entry.peer_id,
            addressesLength: entry.addrs.length,
            addrs: entry.addrs,
            key: entry.key,
            hexDistance: entry.dist,
            libp2p: entry.libp2p,
            binaryDistance,
            xorDistance: entry.key === response.this_key ? '-' : this.getNumberOfZerosUntilFirst1(binaryDistance),
            bucketIndex: response.buckets.indexOf(bucket),
            bucketMaxHex: bucket.max_dist,
          } as NetworkNodeDhtPeer;
        });
        return acc.concat(nodes);
      }, []),
      thisKey: response.this_key,
      buckets: response.buckets.map(bucket => ({
        peers: bucket.entries.length,
        bucketMaxHex: bucket.max_dist,
        maxCapacity: 20,
      }) as NetworkNodeDhtBucket),
    };
  }

  private hexToBinary(hex: string): string {
    const decimalValue = BigInt('0x' + hex);
    const binaryString = decimalValue.toString(2);
    return binaryString.padStart(256, '0');
  }

  private getNumberOfZerosUntilFirst1(binaryString: string): number {
    let leadingZeros = 0;
    for (let i = 0; i < binaryString.length; i++) {
      if (binaryString[i] === '0') {
        leadingZeros++;
      } else {
        break;
      }
    }
    return leadingZeros;
  }

  private mapBootstrapStats(response: any): NetworkNodeDhtBootstrapStats[] {
    return Object.keys(response.requests).map(key => ({
      status: key,
      data: response.requests[key],
    }));
  }
}

export interface DhtPeersResponse {
  this_key: string;
  buckets: Bucket[];
}

export interface Bucket {
  max_dist: string;
  entries: Entry[];
}

export interface Entry {
  peer_id: string;
  libp2p: string;
  key: string;
  dist: string;
  addrs: string[];
}
