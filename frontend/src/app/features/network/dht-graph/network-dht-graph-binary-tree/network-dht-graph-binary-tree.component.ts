//@ts-nocheck
import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, ViewChild } from '@angular/core';
import * as d3 from 'd3';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { NetworkNodeDhtService } from '@network/node-dht/network-node-dht.service';
import {
  NetworkNodeDhtPeer,
  NetworkNodeDhtPeerConnectionType,
} from '@shared/types/network/node-dht/network-node-dht.type';
import { EqualDistanceAlgorithm } from '@network/dht-graph/network-dht-graph-binary-tree/equal-distance-algorithm';

export type DhtGraphNode = {
  peer: NetworkNodeDhtPeer;
  children: DhtGraphNode[];
  prevBranch?: 0 | 1;
  id?: number;
};

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
  private margin = { top: 0, right: 0, bottom: 0, left: 0 };
  private width: number;
  private height: number;

  private data = null;
  private nodeWidth = 25;
  private nodeHeight = 25;
  private radius = 8;
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

    const svgG = svg.append('g');

    const zoom = d3.zoom()
      .scaleExtent([0.1, 10])
      .on('zoom', (ev) => svgG.attr('transform', ev.transform));
    svgG.call(zoom);

    const linksG = svgG.append('g')
      .attr('class', 'links');
    // .attr('transform', `translate(${this.margin.left + (this.width / 2)},${this.margin.top})`);
    const nodesG = svgG.append('g')
      .attr('class', 'nodes');
    // .attr('transform', `translate(${this.margin.left + (this.width / 2)},${this.margin.top})`);

    const node = nodesG.selectAll('.node')
      .data(root.descendants()
        .filter(d => d.data.peer.binaryDistance !== '-'),
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
      .attr('stroke-dasharray', d => d.source.x === d.target.x ? '5,5' : undefined)
      .attr('d', d => {
        const source = d.source;
        const target = d.target;

        // Calculate the intersection points between the circle and the line connecting source and target
        const dx = target.x - source.x;
        const dy = target.y - source.y;
        const dr = Math.sqrt(dx * dx + dy * dy);

        let offsetX = 0;
        let offsetY = 0;
        let sourceX = source.x;
        let sourceY = source.y;
        let targetX = target.x;
        let targetY = target.y;

        // If the source or target is a leaf node, we need to move the line to the edge of the circle
        if (d.source.data.peer.binaryDistance !== '-') {
          offsetX = (dx / dr) * 8;
          offsetY = (dy / dr) * 8;
          sourceX = source.x + offsetX;
          sourceY = source.y + offsetY;
        } else if (d.target.data.peer.binaryDistance !== '-') {
          offsetX = (dx / dr) * 8;
          offsetY = (dy / dr) * 8;
          targetX = target.x - offsetX;
          targetY = target.y - offsetY;
        }

        return `M${sourceX},${sourceY}L${targetX},${targetY}`;
      });

    node.append('circle')
      .attr('fill', (d) => {
        if (this.isOrigin(d)) {
          return 'var(--base-surface)';
        }
        switch (d.data.peer.connection) {
          case NetworkNodeDhtPeerConnectionType.Connected:
            return 'var(--success-primary)';
          case NetworkNodeDhtPeerConnectionType.CanConnect:
          case NetworkNodeDhtPeerConnectionType.NotConnected:
            return 'var(--base-tertiary)';
          case NetworkNodeDhtPeerConnectionType.CannotConnect:
            return 'var(--warn-primary)';
          default:
            return 'var(--base-secondary)';
        }
      })
      .attr('stroke', d => this.isOrigin(d) ? 'var(--special-selected-alt-1-primary)' : undefined)
      .attr('r', this.radius);

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
      .data(root.descendants().slice(1).filter(d => d.x !== d.parent.x))
      .enter()
      .append('text')
      .attr('class', 'link-text')
      .attr('fill', 'var(--base-tertiary)')
      .attr('font-size', '10px')
      .attr('text-anchor', 'middle')
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

    const bounds = nodesG.node().getBBox();
    const { width, height } = bounds;

    const scale = Math.min(
      (this.width) / (width),
      (this.height) / (height),
    );

    const translateX = ((this.width - width * scale) / 2) + width / 2 * scale;
    const translateY = (this.height - height * scale) / 2;

    svgG.call(
      zoom.transform,
      d3.zoomIdentity.translate(translateX, translateY).scale(scale * 0.9),
    );

  }

  private isOrigin(d): boolean {
    return d.data.peer.key === this.peers[0].key;
  }
}
