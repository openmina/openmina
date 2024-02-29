import { ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import * as d3 from 'd3';
import { Simulation } from 'd3';
import { delay, filter, tap } from 'rxjs';
import { zoom } from 'd3-zoom';
import { SimulationNodeDatum } from 'd3-force';
import { DashboardSplitsPeer } from '@shared/types/network/splits/dashboard-splits-peer.type';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { DashboardSplitsLink } from '@shared/types/network/splits/dashboard-splits-link.type';
import {
  selectDashboardSplitsActivePeer,
  selectDashboardSplitsOpenSidePanel,
  selectDashboardSplitsPeersAndLinksAndSetsAndFetching,
} from '@network/splits/dashboard-splits.state';
import { DashboardSplitsSet } from '@shared/types/network/splits/dashboard-splits-set.type';
import { DashboardSplitsSetActivePeer } from '@network/splits/dashboard-splits.actions';
import { eigs } from 'mathjs';
import * as math from 'mathjs';

type DashboardSplitsPeerSimulation = DashboardSplitsPeer & SimulationNodeDatum;
type DashboardSplitsLinkSimulation = {
  index?: number,
  source: DashboardSplitsPeerSimulation,
  target: DashboardSplitsPeerSimulation,
}

@Component({
  selector: 'mina-dashboard-splits-graph',
  templateUrl: './dashboard-splits-graph.component.html',
  styleUrls: ['./dashboard-splits-graph.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-minus-xl w-100' },
})
export class DashboardSplitsGraphComponent extends StoreDispatcher implements OnInit {

  @ViewChild('chart', { static: true }) private chart: ElementRef<HTMLDivElement>;
  @ViewChild('tooltip', { static: true }) private tooltipRef: ElementRef<HTMLDivElement>;

  private width: number;
  private height: number;
  private svg: d3.Selection<SVGSVGElement, unknown, HTMLElement, undefined>;
  private tooltip: d3.Selection<HTMLDivElement, unknown, HTMLElement, undefined>;
  private simulation: Simulation<DashboardSplitsPeerSimulation, undefined>;
  private circles: d3.Selection<SVGCircleElement, DashboardSplitsPeerSimulation, SVGGElement, undefined>;
  private triangles: d3.Selection<SVGPathElement, DashboardSplitsPeerSimulation, SVGGElement, undefined>;
  private squares: d3.Selection<SVGRectElement, DashboardSplitsPeerSimulation, SVGGElement, undefined>;
  private diamonds: d3.Selection<SVGPathElement, DashboardSplitsPeerSimulation, SVGGElement, undefined>;
  private hoveredConnectedLinks: d3.Selection<SVGLineElement, DashboardSplitsLinkSimulation, SVGGElement, undefined>;
  private clickedConnectedLinks: d3.Selection<SVGLineElement, DashboardSplitsLinkSimulation, SVGGElement, undefined>;
  private connections: d3.Selection<SVGLineElement, DashboardSplitsLinkSimulation, SVGGElement, undefined>;
  private zoom: d3.ZoomBehavior<Element, unknown>;
  private activePeer: DashboardSplitsPeer;
  private peers: DashboardSplitsPeer[] = [];
  private links: DashboardSplitsLink[] = [];
  private fetching: boolean = true;

  ngOnInit(): void {
    this.listenToActivePeerChanges();
    this.listenToPeersLinksSets();
    this.listenToSidePanelToggling();
    this.width = this.chart.nativeElement.offsetWidth;
    this.height = this.chart.nativeElement.offsetHeight;
    this.tooltip = d3.select(this.tooltipRef.nativeElement);
    this.svg = d3.select(this.chart.nativeElement)
      .append('svg')
      .attr('width', this.width)
      .attr('height', this.height);

    this.zoom = zoom()
      .scaleExtent([0.1, 10])
      .on('zoom', (event: any) => {
        const { transform } = event;
        this.svg.selectAll('g').attr('transform', transform);
      });
    this.svg.call(this.zoom as any);
  }

  private listenToSidePanelToggling(): void {
    this.select(selectDashboardSplitsOpenSidePanel, () => {
      this.svg?.attr('width', this.chart.nativeElement.offsetWidth);
    }, delay(350));
  }

  private listenToPeersLinksSets() {
    this.select(selectDashboardSplitsPeersAndLinksAndSetsAndFetching, async ({ peers, links, sets, fetching }) => {
        this.fetching = fetching;
        this.peers = peers;
        this.links = links;

        // Step 1: Determine the number of vertices
        const addresses: string[] = [
          ...new Set([
            ...links.flatMap((link: any) => [link.source, link.target]),
            ...peers.map((node: any) => node.address),
          ]),
        ];
        const numVertices: number = addresses.length;

        // Step 2: Initialize the adjacency matrix
        const adjacencyMatrix: number[][] = Array.from({ length: numVertices }, () =>
          Array.from({ length: numVertices }, () => 0)
        );

        // Step 3: Populate the adjacency matrix
        links.forEach((link: any) => {
          const sourceIndex = addresses.indexOf(link.source);
          const targetIndex = addresses.indexOf(link.target);

          // Check if source and target addresses exist in the data array
          if (sourceIndex !== -1 && targetIndex !== -1) {
            adjacencyMatrix[sourceIndex][targetIndex] = 1;
            adjacencyMatrix[targetIndex][sourceIndex] = 1;
          }
        });

        const degreeMatrix: number[][] = Array.from({ length: numVertices }, () =>
          Array.from({ length: numVertices }, () => 0)
        );

        adjacencyMatrix.forEach((row, rowIndex) => {
          const degree = row.reduce((sum, value) => sum + value, 0);
          degreeMatrix[rowIndex][rowIndex] = degree;
        });

        const laplacianMatrix: number[][] = degreeMatrix.map((row, i) =>
          row.map((val, j) => (i === j ? row.reduce((acc, cur, k) => acc + adjacencyMatrix[i][k], 0) : (adjacencyMatrix[i][j] === 0 ? 0 : -adjacencyMatrix[i][j]))),
        );
        console.log('Laplacian Matrix:', laplacianMatrix);

        // check is same as localstorage
        let item = localStorage.getItem('laplacianMatrix');
        if (item) {
          let storedMatrix = JSON.parse(item);
          let storedMatrixString = JSON.stringify(storedMatrix);
          let laplacianMatrixString = JSON.stringify(laplacianMatrix);
          if (storedMatrixString === laplacianMatrixString) {
            console.log('Laplacian Matrix is the same as the stored one');
          } else {
            console.log('Laplacian Matrix is different from the stored one');
          }
        }
        localStorage.setItem('laplacianMatrix', JSON.stringify(laplacianMatrix));

        const computeEigenvalues = async (matrix: number[][]): Promise<number[]> => {
          return eigs(matrix).values as number[];
        };
// Step 2: Compute the eigenvalues of the Laplacian matrix
        const eigenvalues: number[] = await computeEigenvalues(laplacianMatrix); // Use an appropriate method to compute eigenvalues

        console.log('Eigenvalues:', eigenvalues);
// Step 3: Sort the eigenvalues
        const sortedEigenvalues: number[] = eigenvalues.sort((a, b) => a - b);

// Step 4: Calculate the spectral gap
        const spectralGap: number = sortedEigenvalues[1] - sortedEigenvalues[0];

        console.log('Spectral Gap:', spectralGap);

        this.renderGraph({ peers, links, sets });
      }, tap(({ fetching }) => {
        if (fetching) {
          this.fetching = fetching;
        }
      }),
      filter(({ peers, links, fetching }) => peers.length > 0 && links.length > 0 && (this.fetching && !fetching)),
    );
  }

  private listenToActivePeerChanges(): void {
    this.select(selectDashboardSplitsActivePeer, (activePeer: DashboardSplitsPeer) => {
      if (activePeer) {
        this.highlightPeer(activePeer);
      } else {
        this.removeHighlight();
      }
      this.activePeer = activePeer;
    });
  }

  private highlightPeer(activePeer: DashboardSplitsPeer): void {
    this.removeHighlight();
    const addNewColors = (selection: d3.Selection<any, DashboardSplitsPeerSimulation, SVGGElement, undefined>) => {
      selection.filter((peer: DashboardSplitsPeerSimulation) => peer.address === activePeer.address)
        .attr('fill', 'var(--selected-primary)')
        .attr('stroke', (d: DashboardSplitsPeerSimulation) => `var(--${this.links.some(link => link.source === d.address || link.target === d.address) ? 'selected' : 'warn'}-primary)`);
    };
    addNewColors(this.circles);
    addNewColors(this.triangles);
    addNewColors(this.squares);
    addNewColors(this.diamonds);
    this.clickedConnectedLinks = this.connections.filter((link: DashboardSplitsLinkSimulation) => link.source.address === activePeer.address || link.target.address === activePeer.address);
    this.clickedConnectedLinks.attr('stroke', 'var(--selected-primary)');
  }

  private removeHighlight(): void {
    if (!this.activePeer) {
      return;
    }
    const addInitialColors = (selection: d3.Selection<any, DashboardSplitsPeerSimulation, SVGGElement, undefined>) => {
      selection.filter((peer: DashboardSplitsPeerSimulation) => peer.address === this.activePeer.address)
        .attr('fill', 'var(--special-node)')
        .attr('stroke', (d: DashboardSplitsPeerSimulation) => `var(--${this.links.some(link => link.source === d.address || link.target === d.address) ? 'success' : 'warn'}-primary)`);
    };
    addInitialColors(this.circles);
    addInitialColors(this.triangles);
    addInitialColors(this.squares);
    addInitialColors(this.diamonds);
    this.clickedConnectedLinks.attr('stroke', 'var(--base-divider)');
  }

  zoomIn(): void {
    this.svg.transition().duration(500).call(this.zoom.scaleBy as any, 1.5);
  }

  zoomOut(): void {
    this.svg.transition().duration(500).call(this.zoom.scaleBy as any, 0.5);
  }

  zoomReset(): void {
    this.svg.transition().duration(500).call(this.zoom.transform as any, d3.zoomIdentity);
  }

  private renderGraph({ peers, links, sets }: { peers: DashboardSplitsPeer[], links: DashboardSplitsLink[], sets: DashboardSplitsSet[] }): void {
    const nodes: DashboardSplitsPeerSimulation[] = peers.map(peer => ({ ...peer }));
    let lines: DashboardSplitsLinkSimulation[] = [];
    links.forEach(link => {
      const line = {
        source: nodes.find(node => node.address === link.source),
        target: nodes.find(node => node.address === link.target),
      };
      const mutualConnection = lines.some(l => l.source.address === line.target.address && l.target.address === line.source.address);
      const sameConnection = lines.some(l => l.source.address === line.source.address && l.target.address === line.target.address);
      if (!mutualConnection && !sameConnection) {
        lines.push(line);
      }
    });
    this.createSimulation(sets, nodes, lines);

    this.connections = this.getG('links', 'line')
      .selectAll('line')
      .data(lines)
      .enter()
      .append('line')
      .attr('class', d => `link ${d.source.address} ${d.target.address}`)
      .attr('stroke', 'var(--base-divider)');

    const circleNodes = nodes.filter(peer => !peer.node || ['node', 'generator'].some(n => peer.node.toLowerCase().includes(n)));
    const triangleNodes = nodes.filter(p => p.node).filter(peer => peer.node.toLowerCase().includes('snark'));
    const squareNodes = nodes.filter(p => p.node).filter(peer => peer.node.toLowerCase().includes('prod'));
    const diamondNodes = nodes.filter(p => p.node).filter(peer => peer.node.toLowerCase().includes('seed'));

    this.circles = this.getG('circles', 'circle')
      .attr('class', 'circles')
      .selectAll('circle')
      .data(circleNodes)
      .enter()
      .append('circle')
      .attr('r', (d: DashboardSplitsPeerSimulation) => d.radius);
    this.addCommonProperties(this.circles, lines);

    this.triangles = this.getG('triangles', 'path')
      .attr('class', 'triangles')
      .selectAll('path')
      .data(triangleNodes)
      .enter()
      .append('path')
      .attr('d', d3.symbol().type(d3.symbolTriangle2).size(d => d.radius * 20));
    this.addCommonProperties(this.triangles, lines);

    this.squares = this.getG('squares', 'rect')
      .attr('class', 'squares')
      .selectAll('rect')
      .data(squareNodes)
      .enter()
      .append('rect')
      .attr('width', (d: DashboardSplitsPeerSimulation) => d.radius * 2)
      .attr('height', (d: DashboardSplitsPeerSimulation) => d.radius * 2);
    this.addCommonProperties(this.squares, lines);

    this.diamonds = this.getG('diamonds', 'path')
      .attr('class', 'diamonds')
      .selectAll('path')
      .data(diamondNodes)
      .enter()
      .append('path')
      .attr('d', d3.symbol().type(d3.symbolDiamond2).size(d => d.radius * 20));
    this.addCommonProperties(this.diamonds, lines);
  }

  private createSimulation(sets: DashboardSplitsSet[], nodes: DashboardSplitsPeerSimulation[], lines: DashboardSplitsLinkSimulation[]): void {
    const numberOfSets = sets.length;
    const matrixSize = Math.ceil(Math.sqrt(numberOfSets));
    this.simulation = d3.forceSimulation<DashboardSplitsPeerSimulation>(nodes)
      .force('link', d3.forceLink(lines).distance(numberOfSets === 1 ? 250 : 100))  // This adds links between nodes and sets the distance between them
      .force('charge', d3.forceManyBody().strength(-30))  // control the repulsion between groups - negative means bigger distance
      .force('x', d3.forceX<DashboardSplitsPeerSimulation>().x((d: DashboardSplitsPeerSimulation) => {
        for (let i = 0; i < numberOfSets; i++) {
          if (sets[i].peers.some(p => p.address === d.address)) {
            return (i % matrixSize) * this.width / matrixSize;
          }
        }
        return this.width;
      }).strength(0.05))
      .force('y', d3.forceY<DashboardSplitsPeerSimulation>().y((d: DashboardSplitsPeerSimulation) => {
        for (let i = 0; i < numberOfSets; i++) {
          if (sets[i].peers.some(p => p.address === d.address)) {
            return Math.floor(i / matrixSize) * this.height / matrixSize;
          }
        }
        return this.height;
      }).strength(0.05))
      .force('collide', d3.forceCollide().radius(numberOfSets < 3 ? 25 : 17)) // This adds repulsion between nodes. Play with the radius
      .force('center', d3.forceCenter(this.width / 2, this.height / 2))
      .force('bounds', () => {
        for (let node of nodes) {
          const minAllowedX = node.x < 0 ? -node.x : node.x;
          const minAllowedY = node.y < 0 ? -node.y : node.y;
          const maxAllowedX = node.x > this.width ? this.width - (node.x - this.width) : node.x;
          const maxAllowedY = node.y > this.height ? this.height - (node.y - this.height) : node.y;
          node.x = Math.min(maxAllowedX, minAllowedX);
          node.y = Math.min(maxAllowedY, minAllowedY);
        }
      });

    this.simulation.on('tick', () => {
      this.connections.attr('x1', (d: DashboardSplitsLinkSimulation) => d.source.x)
        .attr('y1', (d: DashboardSplitsLinkSimulation) => d.source.y)
        .attr('x2', (d: DashboardSplitsLinkSimulation) => d.target.x)
        .attr('y2', (d: DashboardSplitsLinkSimulation) => d.target.y);
      this.circles
        .attr('cx', (d: DashboardSplitsPeerSimulation) => d.x)
        .attr('cy', (d: DashboardSplitsPeerSimulation) => d.y);
      this.triangles
        .attr('transform', (d: DashboardSplitsPeerSimulation) => `translate(${d.x}, ${d.y})`);
      this.squares
        .attr('x', (d: DashboardSplitsPeerSimulation) => d.x - d.radius)
        .attr('y', (d: DashboardSplitsPeerSimulation) => d.y - d.radius);
      this.diamonds
        .attr('transform', (d: DashboardSplitsPeerSimulation) => `translate(${d.x}, ${d.y})`);
    });

    for (let i = 0; i < 20; ++i) {
      this.simulation.tick();
    }
    this.simulation.alpha(0.1); // controls how much the animation lasts
  }

  private addCommonProperties(selection: d3.Selection<any, DashboardSplitsPeerSimulation, SVGGElement, undefined>, lines: DashboardSplitsLinkSimulation[]): void {
    selection
      .attr('fill', 'var(--special-node)')
      .attr('stroke', (d: DashboardSplitsPeerSimulation) => `var(--${lines.some(link => link.source.address === d.address || link.target.address === d.address) ? 'success' : 'warn'}-primary)`)
      .attr('stroke-width', 1)
      .on('mouseover', (event: MouseEvent & { target: HTMLElement }, peer: DashboardSplitsPeerSimulation) => this.mouseOverHandle(peer, event))
      .on('mouseout', (event: MouseEvent & { target: HTMLElement }, peer: DashboardSplitsPeerSimulation) => this.mouseOutHandler(peer, event))
      .on('click', (event: MouseEvent & { target: HTMLElement }, peer: DashboardSplitsPeerSimulation) => {
        let selectedPeer = this.peers.find(p => p.address === peer.address);
        if (selectedPeer === this.activePeer) {
          selectedPeer = undefined;
        }
        this.dispatch(DashboardSplitsSetActivePeer, selectedPeer);
      });
  }

  private getG(cls: string, subItem: string): d3.Selection<SVGGElement, unknown, HTMLElement, undefined> {
    const selection = this.svg.select<SVGGElement>('g.' + cls);
    if (selection.size() > 0) {
      this.svg.selectAll('g.' + cls + ' ' + subItem).remove();
      return selection;
    }
    return this.svg.append('g')
      .attr('class', cls);
  }

  private mouseOverHandle(peer: DashboardSplitsPeerSimulation, event: MouseEvent & { target: HTMLElement }): void {
    const selection = this.tooltip.html(`${peer.node || peer.address}, <span class="tertiary">→</span> ${peer.incomingConnections} <span class="tertiary">/</span> ${peer.outgoingConnections} <span class="tertiary">→</span>`)
      .style('display', 'block');

    const nodeRect = event.target.getBoundingClientRect();
    const tooltipWidth = selection.node().getBoundingClientRect().width;

    selection
      .style('left', `${nodeRect.left + nodeRect.width / 2 - tooltipWidth / 2}px`)
      .style('top', `${nodeRect.top - 50}px`);

    if (this.activePeer?.address === peer.address) {
      return;
    }
    d3.select(event.target).attr('fill', 'var(--special-node-selected)');
    this.hoveredConnectedLinks = this.connections.filter(link => {
      const isDirectConnection = link.source.address === peer.address || link.target.address === peer.address;
      if (this.activePeer) {
        return isDirectConnection && link.target.address !== this.activePeer.address && link.source.address !== this.activePeer.address;
      }
      return isDirectConnection;
    });

    this.hoveredConnectedLinks.attr('stroke', 'var(--success-primary)');
  }

  private mouseOutHandler(peer: DashboardSplitsPeerSimulation, event: MouseEvent & { target: HTMLElement }): void {
    this.tooltip.style('display', 'none');
    if (this.activePeer?.address === peer.address) {
      return;
    }
    d3.select(event.target).attr('fill', 'var(--special-node)');
    this.hoveredConnectedLinks.attr('stroke', 'var(--base-divider)');
  }
}
