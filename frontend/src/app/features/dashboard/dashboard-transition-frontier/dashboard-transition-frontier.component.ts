import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectDashboardNodes } from '@dashboard/dashboard.state';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { filter } from 'rxjs';

@Component({
  selector: 'mina-dashboard-transition-frontier',
  templateUrl: './dashboard-transition-frontier.component.html',
  styleUrls: ['./dashboard-transition-frontier.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardTransitionFrontierComponent extends StoreDispatcher implements OnInit {

  loading: boolean = true;

  ngOnInit(): void {
    this.listenToNodesChanges();
  }

  private listenToNodesChanges(): void {
    this.select(selectDashboardNodes, (nodes: NodesOverviewNode[]) => {
      this.loading = !nodes[0].ledgers.root.synced;
      this.detect();
    }, filter(n => n.length > 0));
  }
}
