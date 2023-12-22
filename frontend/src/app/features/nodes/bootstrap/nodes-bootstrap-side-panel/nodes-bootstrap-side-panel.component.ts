import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { Router } from '@angular/router';
import { selectNodesBootstrapActiveNode, selectNodesBootstrapNodes } from '@nodes/bootstrap/nodes-bootstrap.state';
import { NodesBootstrapNode } from '@shared/types/nodes/bootstrap/nodes-bootstrap-node.type';
import { NodesBootstrapSetActiveBlock, NodesBootstrapToggleSidePanel } from '@nodes/bootstrap/nodes-bootstrap.actions';
import { Routes } from '@shared/enums/routes.enum';

@Component({
  selector: 'mina-nodes-bootstrap-side-panel',
  templateUrl: './nodes-bootstrap-side-panel.component.html',
  styleUrls: ['./nodes-bootstrap-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 flex-column' },
})
export class NodesBootstrapSidePanelComponent extends StoreDispatcher implements OnInit {

  activeNode: NodesBootstrapNode;
  activeScreen: number = 0;
  activeNodeIndex: number = 0;
  nodesCount: number = 0;

  private nodes: NodesBootstrapNode[] = [];

  constructor(private router: Router) { super(); }

  ngOnInit(): void {
    this.listenToActiveNode();
    this.listenToNodes();
  }

  private listenToActiveNode(): void {
    this.select(selectNodesBootstrapActiveNode, (activeNode: NodesBootstrapNode) => {
      this.activeNode = activeNode;
      if (this.activeNode) {
        this.activeScreen = 1;
        this.getActiveNodeIndex();
      } else {
        this.activeScreen = 0;
      }
      this.detect();
    });
  }

  private listenToNodes(): void {
    this.select(selectNodesBootstrapNodes, (nodes: NodesBootstrapNode[]) => {
      this.nodesCount = nodes.length;
      this.nodes = nodes;
      this.getActiveNodeIndex();
      this.detect();
    });
  }

  private getActiveNodeIndex(): void {
    this.activeNodeIndex = this.nodes.indexOf(this.activeNode);
  }

  toggleSidePanel(): void {
    this.router.navigate([Routes.NODES, Routes.BOOTSTRAP], { queryParamsHandling: 'merge' });
    this.dispatch(NodesBootstrapToggleSidePanel);
  }

  removeActiveNode(): void {
    this.dispatch(NodesBootstrapSetActiveBlock, undefined);
    this.router.navigate([Routes.NODES, Routes.BOOTSTRAP], { queryParamsHandling: 'merge' });
  }
}
