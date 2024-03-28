import { Injectable } from '@angular/core';
import { RustService } from '@core/services/rust.service';
import { map, Observable } from 'rxjs';
import {
  NetworkNodeDhtPeer,
  NetworkNodeDhtPeerConnectionType,
} from '@shared/types/network/node-dht/network-node-dht.type';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

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

  getDhtBootstrapStats(): Observable<NetworkBootstrapStatsRequest[]> {
    return this.rust.get<DhtBootstrapStatsResponse>('/discovery/bootstrap_stats').pipe(
      map((response: DhtBootstrapStatsResponse) => this.mapBootstrapStats(response)),
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
            connection: this.convertConnectionType(entry.connection),
            peerId: entry.peer_id,
            addressesLength: entry.addrs.length,
            addrs: entry.addrs,
            key: entry.key,
            hexDistance: entry.dist,
            libp2p: entry.libp2p,
            binaryDistance,
            bucketIndex: response.buckets.indexOf(bucket),
            xorDistance: entry.key === response.this_key ? '-' : this.getNumberOfZerosUntilFirst1(binaryDistance),
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

  private mapBootstrapStats(response: DhtBootstrapStatsResponse): NetworkBootstrapStatsRequest[] {
    return response.requests;
  }

  private convertConnectionType(connection: ConnectionType): NetworkNodeDhtPeerConnectionType {
    const connectionString = connection.replace(/([A-Z])/g, ' $1').trim();
    return connectionString as NetworkNodeDhtPeerConnectionType;
  }
}

interface DhtPeersResponse {
  this_key: string;
  buckets: Bucket[];
}

interface Bucket {
  max_dist: string;
  entries: Entry[];
}

interface Entry {
  peer_id: string;
  libp2p: string;
  key: string;
  dist: string;
  addrs: string[];
  connection: ConnectionType;
}

enum ConnectionType {
  CannotConnect = 'CannotConnect',
  Connected = 'Connected',
  NotConnected = 'NotConnected',
  CanConnect = 'CanConnect',
}

interface DhtBootstrapStatsResponse {
  requests: NetworkBootstrapStatsRequest[];
}
