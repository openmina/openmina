import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectDashboardNodesAndPeers } from '@dashboard/dashboard.state';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { filter } from 'rxjs';
import { NodesOverviewNodeBlockStatus } from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import { lastItem, ONE_MILLION } from '@openmina/shared';
import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';

const PENDING = 'Pending';
const SYNCED = 'Synced';

@Component({
  selector: 'mina-dashboard-blocks-sync',
  templateUrl: './dashboard-blocks-sync.component.html',
  styleUrls: ['./dashboard-blocks-sync.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardBlocksSyncComponent extends StoreDispatcher implements OnInit {

  fetched: number;
  fetchedPercentage: string = '-';
  applied: number;
  appliedPercentage: number;
  root: number;
  rootText: string = PENDING;
  bestTipBlock: number;
  bestTipBlockSyncedText: string = PENDING;
  targetBlock: number;
  syncProgress: string;

  ngOnInit(): void {
    this.listenToNodesChanges();
  }

  private listenToNodesChanges(): void {
    this.select(selectDashboardNodesAndPeers, ([nodes, peers]: [NodesOverviewNode[], DashboardPeer[]]) => {
      if (nodes.length === 0) {
        this.fetched = undefined;
        this.fetchedPercentage = '-';
        this.applied = undefined;
        this.appliedPercentage = undefined;
        this.root = undefined;
        this.rootText = PENDING;
        this.bestTipBlock = undefined;
        this.bestTipBlockSyncedText = PENDING;
        this.targetBlock = undefined;
        this.syncProgress = undefined;
      } else {
        this.extractNodesData(nodes);
        this.extractPeersData(peers);
      }
      this.detect();
    });
  }

  private extractPeersData(peers: DashboardPeer[]): void {
    const highestHeightPeer = peers.reduce(
      (acc: DashboardPeer, peer: DashboardPeer) => peer.height > acc.height ? peer : acc,
      { height: 0 } as DashboardPeer,
    );
    this.targetBlock = highestHeightPeer.height;
  }

  private extractNodesData(nodes: NodesOverviewNode[]): void {
    let blocks = nodes[0].blocks;

    if (blocks.length > 0) {
      this.bestTipBlock = blocks[0].height;
      this.bestTipBlockSyncedText = 'Fetched ' + this.calculateProgressTime(nodes[0].bestTipReceivedTimestamp * ONE_MILLION).slice(7);
      this.syncProgress = this.bestTipBlockSyncedText.slice(8);
    }

    if (blocks.length === 291) {
      this.root = lastItem(blocks).height;
      this.rootText = this.calculateProgressTime(lastItem(blocks).applyEnd);
      if (blocks[0].status === NodesOverviewNodeBlockStatus.APPLIED) {
        this.bestTipBlockSyncedText = SYNCED + ' ' + this.bestTipBlockSyncedText.slice(7);
      }
    } else {
      this.root = null;
      this.rootText = PENDING;
    }
    blocks = blocks.slice(1);

    this.fetched = blocks.filter(b => ![NodesOverviewNodeBlockStatus.MISSING, NodesOverviewNodeBlockStatus.FETCHING].includes(b.status)).length;
    this.applied = blocks.filter(b => b.status === NodesOverviewNodeBlockStatus.APPLIED).length;
    this.fetchedPercentage = Math.round(this.fetched * 100 / 291) + '%';
    this.appliedPercentage = Math.round(this.applied * 100 / 291);
  }

  private calculateProgressTime(timestamp: number): string {
    if (!timestamp) {
      return 'Pending';
    }
    timestamp = Math.ceil(timestamp / ONE_MILLION);
    const millisecondsAgo = Date.now() - timestamp;
    const minutesAgo = Math.floor(millisecondsAgo / 1000 / 60);
    const hoursAgo = Math.floor(minutesAgo / 60);
    const daysAgo = Math.floor(hoursAgo / 24);

    if (daysAgo > 0) {
      return `${SYNCED} ${daysAgo}d ago`;
    } else if (hoursAgo > 0) {
      return `${SYNCED} ${hoursAgo}h ago`;
    } else if (minutesAgo > 0) {
      return `${SYNCED} ${minutesAgo}m ago`;
    } else {
      return `${SYNCED} <1m ago`;
    }
  }
}
