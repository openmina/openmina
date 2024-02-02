import { ChangeDetectionStrategy, Component, OnInit, TemplateRef, ViewChild } from '@angular/core';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { getMergedRoute, MergedRoute, TableColumnList } from '@openmina/shared';
import { Router } from '@angular/router';
import { NodesOverviewSetActiveNode, NodesOverviewSortNodes } from '@nodes/overview/nodes-overview.actions';
import {
  selectNodesOverviewActiveNode,
  selectNodesOverviewNodes,
  selectNodesOverviewSort,
} from '@nodes/overview/nodes-overview.state';
import { Routes } from '@shared/enums/routes.enum';
import { filter, take } from 'rxjs';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-nodes-overview-table',
  templateUrl: './nodes-overview-table.component.html',
  styleUrls: ['./nodes-overview-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NodesOverviewTableComponent extends MinaTableRustWrapper<NodesOverviewNode> implements OnInit {

  protected readonly tableHeads: TableColumnList<NodesOverviewNode> = [
    { name: 'status', sort: 'kind' },
    { name: 'name' },
    { name: 'height', tooltip: 'The block height on which the Snarker is currently working. ' },
    {
      name: 'best tip',
      sort: 'bestTip',
      tooltip: 'The blockchain\'s latest block with the highest known chain strength.',
    },
    { name: 'datetime', sort: 'bestTipReceivedTimestamp', tooltip: 'The date when the block was received.' },
    {
      name: 'applied',
      sort: 'appliedBlocks',
      tooltip: 'Number of blocks that node has applied with the latest synchronization attempt.',
    },
    {
      name: 'applying',
      sort: 'applyingBlocks',
      tooltip: 'Number of blocks that node is currently applying with the latest synchronization attempt.',
    },
    {
      name: 'fetching',
      sort: 'fetchingBlocks',
      tooltip: 'Number of blocks that node is currently fetching with the latest synchronization attempt.',
    },
    {
      name: 'fetched',
      sort: 'fetchedBlocks',
      tooltip: 'Number of blocks that node has fetched with the latest synchronization attempt.',
    },
    {
      name: 'missing blocks',
      sort: 'missingBlocks',
      tooltip: 'Number of blocks that the node needs to fetch with the latest synchronization attempt.',
    },
  ];

  private nodeFromRoute: string;
  @ViewChild('thGroupsTemplate') private thGroupsTemplate: TemplateRef<void>;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToNodesChanges();
    this.listenToActiveNodeChange();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [100, 180, 80, 130, 165, 120, 120, 120, 120, 120];
    this.table.minWidth = 1335;
    this.table.propertyForActiveCheck = 'name';
    this.table.thGroupsTemplate = this.thGroupsTemplate;
    this.table.sortClz = NodesOverviewSortNodes;
    this.table.sortSelector = selectNodesOverviewSort;
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['node'] && this.table.rows.length === 0) {
        this.nodeFromRoute = route.params['node'];
      }
    }, take(1));
  }

  private listenToNodesChanges(): void {
    this.select(selectNodesOverviewNodes, (nodes: NodesOverviewNode[]) => {
      this.table.rows = nodes;
      this.table.detect();
      if (this.nodeFromRoute) {
        this.scrollToElement();
      }
      this.detect();
    }, filter(nodes => nodes.length > 0));
  }

  private listenToActiveNodeChange(): void {
    this.select(selectNodesOverviewActiveNode, (node: NodesOverviewNode) => {
      this.table.activeRow = node;
      this.table.detect();
      this.detect();
    });
  }

  private scrollToElement(): void {
    const finder = (node: NodesOverviewNode) => node.name === this.nodeFromRoute;
    const i = this.table.rows.findIndex(finder);
    this.table.scrollToElement(finder);
    delete this.nodeFromRoute;
    this.onRowClick(this.table.rows[i]);
  }

  protected override onRowClick(row: NodesOverviewNode): void {
    if (this.table.activeRow?.name !== row?.name) {
      this.dispatch(NodesOverviewSetActiveNode, row);
      this.router.navigate([Routes.NODES, Routes.OVERVIEW, row.name], { queryParamsHandling: 'merge' });
    }
  }
}
