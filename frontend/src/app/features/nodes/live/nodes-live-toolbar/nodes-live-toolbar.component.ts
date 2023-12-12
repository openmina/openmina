import { ChangeDetectionStrategy, Component, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectNodesLiveActiveNode, selectNodesLiveNodes } from '@nodes/live/nodes-live.state';
import { NodesLiveNode } from '@shared/types/nodes/live/nodes-live-node.type';
import { NodesLiveSetActiveNode } from '@nodes/live/nodes-live.actions';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { getMergedRoute, MergedRoute } from '@openmina/shared';
import { filter, take } from 'rxjs';

@Component({
  selector: 'mina-nodes-live-toolbar',
  templateUrl: './nodes-live-toolbar.component.html',
  styleUrls: ['./nodes-live-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-lg fx-row-vert-cent' },
})
export class NodesLiveToolbarComponent extends StoreDispatcher implements OnInit {

  node: NodesLiveNode = {} as NodesLiveNode;

  nodes: NodesLiveNode[] = [];
  private tipFromRoute: string;

  constructor(private router: Router) {super();}

  ngOnInit(): void {
    this.listenToBestTip();
    this.listenToNodes();
    this.listenToRoute();
  }

  private listenToRoute(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['bestTip']) {
        this.tipFromRoute = route.params['bestTip'];
      }
    }, take(1));
  }

  private listenToBestTip(): void {
    this.select(selectNodesLiveActiveNode, (node: NodesLiveNode) => {
      this.node = node;
      this.detect();
    }, filter(Boolean));
  }

  selectPreviousNode(): void {
    const index = this.nodes.findIndex((node: NodesLiveNode) => node.bestTip === this.node.bestTip);
    const previous = this.nodes[index - 1];
    if (previous) {
      this.selectNode(previous.bestTip);
    }
  }

  selectNextNode(): void {
    const index = this.nodes.findIndex((node: NodesLiveNode) => node.bestTip === this.node.bestTip);
    const next = this.nodes[index + 1];
    if (next) {
      this.selectNode(next.bestTip);
    }
  }

  selectNode(hash: string): void {
    this.dispatch(NodesLiveSetActiveNode, { hash });
    this.router.navigate([Routes.NODES, Routes.LIVE, hash], { queryParamsHandling: 'merge' });
  }

  private listenToNodes(): void {
    this.select(selectNodesLiveNodes, (nodes: NodesLiveNode[]) => {
      this.nodes = nodes;
      if (this.tipFromRoute) {
        this.selectNode(this.tipFromRoute);
        delete this.tipFromRoute;
      }
      this.detect();
    }, filter(Boolean));
  }

  selectLastTip(): void {
    this.selectNode(this.nodes[this.nodes.length - 1].bestTip);
  }

  selectFirstTip(): void {
    this.selectNode(this.nodes[0].bestTip);
  }
}
