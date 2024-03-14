import { Injectable } from '@angular/core';
import { RustService } from '@core/services/rust.service';
import { map, Observable } from 'rxjs';
import { NetworkNodeDHT } from '@shared/types/network/node-dht/network-node-dht.type';

@Injectable({
  providedIn: 'root'
})
export class NetworkNodeDhtService {

  constructor(private rust: RustService) {
  }

  getDhtPeers(): Observable<{ peers: NetworkNodeDHT[], thisKey: string }> {
    return this.rust.get<DhtPeersResponse>('/discovery/routing_table').pipe(
      map((response: DhtPeersResponse) => this.mapDhtPeers(response))
    );
  }

  private mapDhtPeers(response: DhtPeersResponse): { peers: NetworkNodeDHT[], thisKey: string } {
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
            binaryDistance,
            xorDistance: entry.key === response.this_key ? '-' : this.getNumberOfZerosUntilFirst1(binaryDistance),
            bucketIndex: response.buckets.indexOf(bucket),
            bucketMaxHex: bucket.max_dist
          } as NetworkNodeDHT;
        });
        return acc.concat(nodes);
      }, []),
      thisKey: response.this_key
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
