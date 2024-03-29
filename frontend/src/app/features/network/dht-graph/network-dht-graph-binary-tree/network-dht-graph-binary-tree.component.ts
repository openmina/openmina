//@ts-nocheck
import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, ViewChild } from '@angular/core';
import * as d3 from 'd3';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtService } from '@network/node-dht/network-node-dht.service';
import { NetworkNodeDhtPeer } from '@shared/types/network/node-dht/network-node-dht.type';
import { EqualDistanceAlgorithm } from '@network/dht-graph/network-dht-graph-binary-tree/equal-distance-algorithm';
import { MinimumDistanceAlgorithm } from '@network/dht-graph/network-dht-graph-binary-tree/minimum-distance-algorithm';

export type TreeNode = {
  binaryDistance: string;
  children: TreeNode[];
  prevBranch?: 0 | 1;
  id?: number;
};

const binaryTree: TreeNode = {
  binaryDistance: 1,
  children: [
    {
      binaryDistance: 2,
      children: [
        {
          binaryDistance: 4,
          children: [],
        },
        {
          binaryDistance: 5,
          children: [],
        },
      ],
    },
    {
      binaryDistance: 3,
      children: [
        {
          binaryDistance: 6,
          children: [],
        },
        {
          binaryDistance: 7,
          children: [],
        },
      ],
    },
  ],
};

// class TreeNode2 {
//   peer: NetworkNodeDhtPeer;
//   children: TreeNode2[] = [];
//
//   constructor(peer: NetworkNodeDhtPeer) {
//     this.peer = { binaryDistance: peer.binaryDistance };
//   }
// }

@Component({
  selector: 'mina-network-dht-graph-binary-tree',
  templateUrl: './network-dht-graph-binary-tree.component.html',
  styleUrls: ['./network-dht-graph-binary-tree.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 flex-column' },
})
export class NetworkDhtGraphBinaryTreeComponent extends StoreDispatcher implements AfterViewInit {
  @ViewChild('graph') private graph: ElementRef<HTMLDivElement>;

  // Define margins as a property of the class
  private margin = { top: 150, right: 20, bottom: 20, left: 20 };
  private width: number;
  private height: number;

  private data = null;
  private nodeWidth = 60;
  private nodeHeight = 30;
  private peers: NetworkNodeDhtPeer[] = [];

  constructor(private nodeDhtService: NetworkNodeDhtService) { super(); }

  ngAfterViewInit(): void {
    this.nodeDhtService.getDhtPeers().subscribe(({ peers }) => {
      const sortPeers = (a: NetworkNodeDhtPeer, b: NetworkNodeDhtPeer) => {
        const aBinaryDistance = a.binaryDistance;
        const bBinaryDistance = b.binaryDistance;
        for (let i = 0; i < aBinaryDistance.length; i++) {
          if (aBinaryDistance[i] === bBinaryDistance[i]) {
            continue;
          }
          return aBinaryDistance[i] === '1' ? 1 : -1;
        }
        return 0;
      };
      let newPeers = peers.slice().sort(sortPeers).map((p, i) => ({ ...p, id: i }));
      // newPeers = newPeers.slice(0, 8).map(p => ({ binaryDistance: p.binaryDistance.slice(0, 8) }));
      // newPeers = ['000', '100', '110', '111'].map(p => ({ binaryDistance: p }));
      this.peers = newPeers;
      // console.log(newPeers.map(p => ({ binaryDistance: p.binaryDistance })));
      EqualDistanceAlgorithm.peers = newPeers;
      this.data = EqualDistanceAlgorithm.createBinaryTree(newPeers);
      this.createGraph();
    });
  }

  private createGraph(): void {
    this.width = this.graph.nativeElement.clientWidth;
    this.height = this.graph.nativeElement.clientHeight;

    const root = d3.hierarchy(this.data, (d) => d ? d.children : null);
    const treeLayout = d3.tree().nodeSize([this.nodeWidth, this.nodeHeight]);
    treeLayout(root);

    const svg = d3.select(this.graph.nativeElement)
      .append('svg')
      .attr('width', this.width + this.margin.left + this.margin.right)
      .attr('height', this.height + this.margin.top + this.margin.bottom);

    const linksG = svg.append('g')
      .attr('class', 'links')
      .attr('transform', `translate(${this.margin.left + (this.width / 2)},${this.margin.top})`);
    const nodesG = svg.append('g')
      .attr('class', 'nodes')
      .attr('transform', `translate(${this.margin.left + (this.width / 2)},${this.margin.top})`);

    const node = nodesG.selectAll('.node')
      .data(root.descendants()
        .filter(d => d.data.binaryDistance !== '-'),
      )
      .enter()
      .append('g')
      .attr('class', 'node')
      .attr('transform', d => `translate(${d.x},${d.y})`)
      .on('click', (ev, d) => console.log(d.data));

    // const link = linksG.selectAll('.link')
    //   .data(root.descendants().slice(1))
    //   .enter()
    //   .append('path')
    //   .attr('class', 'link')
    //   .attr('fill', 'none')
    //   .attr('stroke', 'var(--base-tertiary)')
    //   .attr('d', function (d) {
    //     return 'M' + d.x + ',' + d.y
    //       + 'V' + d.parent.y
    //       + 'H' + d.parent.x;
    //   });
    const link = linksG.selectAll('.link')
      .data(root.links())
      .enter()
      .append('path')
      .attr('class', 'link')
      .attr('fill', 'none')
      .attr('stroke', 'var(--base-tertiary)')
      .attr('d', d => d3.line()
        .x(d => d.x)
        .y(d => d.y)([d.source, d.target]));

    node.append('circle')
      .attr('fill', 'var(--base-surface)')
      .attr('stroke', 'var(--base-tertiary)')
      .attr('r', 16);

    // node.append('text')
    //   .attr('dy', '.35em')
    //   .style('text-anchor', 'middle')
    //   .attr('fill', 'var(--base-primary)')
    //   .attr('font-size', '12px')
    //   .attr('font-weight', '900')
    //   .text(d => d.data.binaryDistance === '-' ? null : d.data.binaryDistance.slice(0, 8));

    // Root is placed in a special way. We need to move it up and draw a line to it
    // const rootNode = nodesG.selectAll('.node').filter(d => d.parent === null);
    // rootNode.attr('transform', d => 'translate(' + d.x + ',' + (d.y - this.nodeHeight) + ')');
    // linksG.append('line')
    //   .attr('x1', root.x)
    //   .attr('y1', root.y - this.nodeHeight)
    //   .attr('x2', root.x)
    //   .attr('y2', root.y)
    //   .attr('fill', 'none')
    //   .attr('stroke', 'var(--base-tertiary)');
    //
    // rootNode.raise();

    // Add numbers above the horizontal lines to indicate left (0) or right (1) side
    const linkText = linksG.selectAll('.link-text')
      .data(root.descendants().slice(1))
      .enter()
      .append('text')
      .attr('class', 'link-text')
      .attr('fill', 'var(--base-primary)')
      .attr('font-size', '10px')
      .attr('text-anchor', 'middle')
      // .attr('x', d => (d.x + d.parent.x) / 2)
      // .attr('y', d => d.y - this.nodeHeight - 7)
      .attr('x', d => {
        if (d.x === d.parent.x) {
          return d.data.prevBranch === 0 ? d.x - 10 : d.x + 10;
        }
        return (d.x + d.parent.x) / 2 + (d.x > d.parent.x ? 10 : -10);
      })
      .attr('y', d => {
        if (d.x === d.parent.x) {
          return d.y - (this.nodeHeight / 2);
        }
        return (d.y + this.nodeHeight / 2) - this.nodeHeight + (d.y < d.parent.y ? 2 : -2);
      })
      .text(d => d.data.prevBranch);

    const zoom = d3.zoom()
      .scaleExtent([0.1, 10])
      .on('zoom', (ev) => svg.attr('transform', ev.transform));
    // svg.call(zoom);

    //center the tree as it is not fully balanced left-right
    // const treeWidth = linksG.node().getBBox().width;
    // const svgWidth = this.width + this.margin.left + this.margin.right;
    // nodesG.attr('transform', `translate(${(svgWidth - treeWidth) / 2},${this.margin.top})`);
    // linksG.attr('transform', `translate(${(svgWidth - treeWidth) / 2},${this.margin.top})`);
  }
}
