import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  selectNetworkNodeDhtActivePeer,
  selectNetworkNodeDhtKeyPeersBucketsOpenSidePanel,
} from '@network/node-dht/network-node-dht.state';
import { filter, fromEvent, skip, tap } from 'rxjs';
import { NetworkNodeDhtSetActivePeer } from '@network/node-dht/network-node-dht.actions';
import { NetworkNodeDhtBucket } from '@shared/types/network/node-dht/network-node-dht-bucket.type';
import {
  NetworkNodeDhtPeer,
  NetworkNodeDhtPeerConnectionType,
} from '@shared/types/network/node-dht/network-node-dht.type';
import { untilDestroyed } from '@ngneat/until-destroy';

interface DhtPoint {
  left: number;
  isBucket: boolean;
  distance?: number;
  isOrigin?: boolean;
  peerId?: string;
  originalLeft?: number;
  connection?: NetworkNodeDhtPeerConnectionType;
}

@Component({
  selector: 'mina-network-node-dht-line',
  templateUrl: './network-node-dht-line.component.html',
  styleUrls: ['./network-node-dht-line.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column w-100 pl-12 pr-12' },
})
export class NetworkNodeDhtLineComponent extends StoreDispatcher implements AfterViewInit {

  activePeer: NetworkNodeDhtPeer;
  points: DhtPoint[] = [];
  bucketPoints: DhtPoint[] = [];
  openSidePanel: boolean;
  buckets: NetworkNodeDhtBucket[];

  private thisKey: string;
  private peers: NetworkNodeDhtPeer[] = [];
  private lastKnownWidth: number;

  readonly trackPoints = (index: number) => index;
  readonly trackBuckets = (index: number) => index;

  @ViewChild('line') private line: ElementRef<HTMLDivElement>;

  ngAfterViewInit(): void {
    this.listenToNodeDhtPeers();
    this.listenToActiveNodeDhtPeer();
    this.listenToElWidthChange();
  }

  private listenToElWidthChange(): void {
    this.lastKnownWidth = this.line.nativeElement.offsetWidth;
    const widthChange = 'width-change';
    const resizeObserver = new ResizeObserver((entries: ResizeObserverEntry[]) => {
      for (const entry of entries) {
        const widthChangeEvent = new CustomEvent(widthChange);
        entry.target.dispatchEvent(widthChangeEvent);
      }
    });
    resizeObserver.observe(this.line.nativeElement);

    fromEvent(this.line.nativeElement, widthChange).pipe(
      filter(() => !!this.thisKey),
      filter(() => this.lastKnownWidth !== this.line.nativeElement.offsetWidth),
      tap(() => {
        this.lastKnownWidth = this.line.nativeElement.offsetWidth;
        this.calculateLeftBasedOnParentContainer();
        this.detect();
      }),
      untilDestroyed(this),
    ).subscribe();
  }

  private listenToNodeDhtPeers(): void {
    this.select(selectNetworkNodeDhtKeyPeersBucketsOpenSidePanel, ([key, peers, buckets, openSidePanel]: [string, NetworkNodeDhtPeer[], NetworkNodeDhtBucket[], boolean]) => {
      this.openSidePanel = openSidePanel;
      this.thisKey = key;
      if (
        this.thisKey
        && (this.peers.length === 0 || this.peers.some((p, i) => p.hexDistance !== peers[i].hexDistance))
      ) {
        this.peers = peers;
        this.buckets = buckets;
        this.calculate();
      }
      this.detect();
    }, filter(data => !!data[0]));
  }

  private calculateLeftBasedOnParentContainer(): void {
    const max = this.line.nativeElement.offsetWidth - 16;
    this.points.forEach(point => point.left = (point.originalLeft / 100) * max);
  }

  private calculate(): void {
    this.points = [];
    this.bucketPoints = [];
    const max_keyspace_hex = this.getMaxOfHex(this.buckets);
    const max_keyspace_int = BigInt('0x' + max_keyspace_hex);

    const buckets = this.buckets;
    for (const bucket of buckets.slice().reverse()) {
      const this_bucket_key_int = BigInt('0x' + bucket.bucketMaxHex);
      const left_percent = (Number(this_bucket_key_int) / Number(max_keyspace_int)) * 100;
      this.bucketPoints.push({
        left: left_percent,
        isBucket: true,
        peerId: '',
      });
    }

    for (const peer of this.peers) {
      const dist_int = BigInt('0x' + peer.hexDistance);
      const dist_normalized = (Number(dist_int) / Number(max_keyspace_int));
      this.points.push({
        left: null,
        connection: peer.connection,
        originalLeft: dist_normalized * 100,
        isBucket: false,
        isOrigin: peer.key === this.thisKey,
        peerId: peer.peerId,
      });
    }
    this.calculateLeftBasedOnParentContainer();
  }

  private getMaxOfHex(buckets: NetworkNodeDhtBucket[]): string {
    return buckets.reduce((maxHex, bucket) => maxHex > bucket.bucketMaxHex ? maxHex : bucket.bucketMaxHex, '0');
  }

  private listenToActiveNodeDhtPeer(): void {
    this.select(selectNetworkNodeDhtActivePeer, (peer: NetworkNodeDhtPeer) => {
      this.activePeer = peer;
      this.detect();
    }, skip(1));
  }

  selectPeer(point: DhtPoint): void {
    this.dispatch(NetworkNodeDhtSetActivePeer, this.peers.find(p => p.peerId === point.peerId));
  }
}
