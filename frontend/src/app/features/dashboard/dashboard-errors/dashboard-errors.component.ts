import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectDashboardNodes } from '@dashboard/dashboard.state';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { ONE_MILLION } from '@openmina/shared';
import { filter } from 'rxjs';
import {
  NodesOverviewResync,
  NodesOverviewResyncKindType,
  NodesOverviewResyncUI,
} from '@shared/types/nodes/dashboard/nodes-overview-resync.type';

const descriptionMap = {
  [NodesOverviewResyncKindType.RootLedgerChange]: 'Root snarked ledger needs to be re-synced.',
  [NodesOverviewResyncKindType.FetchStagedLedgerError]: 'Root staging ledger needs to be re-synced.',
  [NodesOverviewResyncKindType.EpochChange]: 'Next epoch ledger needs to be re-synced.',
  [NodesOverviewResyncKindType.BestChainChange]: 'Staking epoch ledger needs to be re-synced.',
};

@Component({
  selector: 'mina-dashboard-errors',
  templateUrl: './dashboard-errors.component.html',
  styleUrls: ['./dashboard-errors.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardErrorsComponent extends StoreDispatcher implements OnInit {

  resyncs: NodesOverviewResyncUI[] = [];
  open: boolean;
  readonly trackResyncs = (_: number, resync: NodesOverviewResyncUI) => resync.kind + resync.timeAgo;

  ngOnInit(): void {
    this.listenToNodesChanges();
  }

  private listenToNodesChanges(): void {
    this.select(selectDashboardNodes, (nodes: NodesOverviewNode[]) => {
      if (this.resyncs.length === 0 && nodes[0].resyncs.length > 0) {
        this.open = true;
      }
      this.mapResyncs(nodes[0].resyncs);
      this.detect();
    }, filter(n => n.length > 0));
  }

  private mapResyncs(resyncs: NodesOverviewResync[]): void {
    this.resyncs = resyncs.slice().reverse().map(resync => ({
      ...resync,
      description: resync.description ?? descriptionMap[resync.kind],
      timeAgo: this.calculateProgressTime(resync.time),
    } as NodesOverviewResyncUI));
  }

  private calculateProgressTime(timestamp: number): string {
    timestamp = Math.ceil(timestamp / ONE_MILLION);
    const millisecondsAgo = Date.now() - timestamp;
    const minutesAgo = Math.floor(millisecondsAgo / 1000 / 60);
    const hoursAgo = Math.floor(minutesAgo / 60);
    const daysAgo = Math.floor(hoursAgo / 24);

    if (daysAgo > 0) {
      return `${daysAgo}d ago`;
    } else if (hoursAgo > 0) {
      return `${hoursAgo}h ago`;
    } else if (minutesAgo > 0) {
      return `${minutesAgo}m ago`;
    } else {
      return `<1m ago`;
    }
  }
}
