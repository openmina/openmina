import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { DashboardGetData, DashboardInit } from '@dashboard/dashboard.actions';
import { filter, skip, tap, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { AppSelectors } from '@app/app.state';
import { selectDashboardNodesAndPeers, selectDashboardNodesAndRpcStats, selectDashboardPeersStats } from '@dashboard/dashboard.state';
import { DashboardPeersStats } from '@shared/types/dashboard/dashboard-peers-stats.type';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';
import { NodesOverviewLedgerStepState } from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { lastItem, ONE_MILLION, SecDurationConfig } from '@openmina/shared';
import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';
import { NodesOverviewNodeBlockStatus } from '@shared/types/nodes/dashboard/nodes-overview-block.type';

@Component({
  selector: 'mina-dashboard',
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 w-100 flex-column' },
})
export class DashboardComponent extends StoreDispatcher implements OnInit {

  updateAction: string;
  updateDetail: string;
  connectedProgress: number = 0;
  ledgerProgress: number = 0;
  blockSyncProgress: number = 0;

  private connected: boolean = false;

  ngOnInit(): void {
    this.updateAction = 'Connecting to peers';

    this.listenToNodeChanging();
    this.listenToPeersChanges();
    this.getDashboardData();
    this.listenToNodesChanges();
    this.listenToNodesChanges();
  }

  private getDashboardData(): void {
    this.dispatch(DashboardInit);
    timer(4000, 4000)
      .pipe(
        tap(() => this.dispatch(DashboardGetData)),
        untilDestroyed(this),
      )
      .subscribe();
  }

  private listenToNodeChanging(): void {
    this.select(AppSelectors.activeNode, () => {
      this.dispatch(DashboardGetData, { force: true });
    }, filter(Boolean), skip(1));
  }

  private listenToPeersChanges(): void {
    this.select(selectDashboardPeersStats, (stats: DashboardPeersStats) => {
      if (this.connected && stats.connected === 0 || !this.connected) {
        if (stats.connected > 0) {
          this.updateAction = `Connected to ${stats.connected} peer${stats.connected !== 1 ? 's' : ''}`;
        } else if (stats.connecting > 0) {
          this.updateAction = `Connecting to ${stats.connecting} peer${stats.connecting !== 1 ? 's' : ''}`;
        } else {
          this.updateAction = 'Looking for peers';
        }
        if (stats.connected) {
          this.connectedProgress = 33;
        } else {
          this.connectedProgress = 0;
          this.ledgerProgress = 0;
          this.blockSyncProgress = 0;
        }
        this.detect();
      }
      this.connected = stats.connected > 0;
    }, skip(1));
  }

  private listenToNodesChanges(): void {
    this.select(selectDashboardNodesAndRpcStats, ([nodes, rpcStats]: [NodesOverviewNode[], DashboardRpcStats]) => {
      if (nodes.length !== 0) {
        const ledgers = nodes[0].ledgers;

        let stakingProgress = rpcStats.stakingLedger?.fetched / rpcStats.stakingLedger?.estimation * 100 || 0;
        let nextProgress = rpcStats.nextLedger?.fetched / rpcStats.nextLedger?.estimation * 100 || 0;
        let rootSnarkedProgress = rpcStats.rootLedger?.fetched / rpcStats.rootLedger?.estimation * 100 || 0;
        let rootStagedProgress = ledgers.rootStaged.staged.fetchPartsEnd ? 50 : 0;

        if (ledgers.stakingEpoch.state === NodesOverviewLedgerStepState.SUCCESS) {
          stakingProgress = 100;
          this.updateAction = 'Downloading Next Epoch Ledger';
        }
        if (ledgers.nextEpoch.state === NodesOverviewLedgerStepState.SUCCESS) {
          nextProgress = 100;
          this.updateAction = 'Downloading Root Snarked Ledger';
        }
        if (ledgers.rootSnarked.state === NodesOverviewLedgerStepState.SUCCESS) {
          rootSnarkedProgress = 100;
          this.updateAction = 'Downloading Root Staged Ledger';
        }
        if (ledgers.rootStaged.state === NodesOverviewLedgerStepState.SUCCESS) {
          rootStagedProgress = 100;
          this.updateAction = 'Fetching Blocks';
        }
        const ledgerProgressTotal = (stakingProgress + nextProgress + rootSnarkedProgress + rootStagedProgress) / 4;
        this.ledgerProgress = ledgerProgressTotal * 0.33;

        let blocks = nodes[0].blocks;

        blocks = blocks.slice(1);

        const fetched = blocks.filter(b => ![NodesOverviewNodeBlockStatus.MISSING, NodesOverviewNodeBlockStatus.FETCHING].includes(b.status)).length;
        const fetchedPercentage = Math.round(fetched * 100 / 291);

        const applied = blocks.filter(b => b.status === NodesOverviewNodeBlockStatus.APPLIED).length;
        const appliedPercentage = Math.round(applied * 100 / 291);
        this.blockSyncProgress = appliedPercentage * 0.34;

        if (fetchedPercentage < 100) {
          this.updateAction = `Fetching Blocks (${fetchedPercentage}%)`;
        } else if (appliedPercentage < 100) {
          this.updateAction = `Applying Blocks (${appliedPercentage}%)`;
        } else {
          this.updateAction = '';
        }

      } else {
        this.ledgerProgress = 0;
        this.blockSyncProgress = 0;
      }
      this.detect();
    });
  }
}
