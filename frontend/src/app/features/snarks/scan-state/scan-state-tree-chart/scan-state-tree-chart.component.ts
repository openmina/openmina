import { ChangeDetectionStrategy, Component, ElementRef, Input, OnChanges, OnInit, ViewChild } from '@angular/core';
import * as d3 from 'd3';
import { ScanStateTree } from '@shared/types/snarks/scan-state/scan-state-tree.type';
import { ScanStateLeaf, ScanStateLeafStatus } from '@shared/types/snarks/scan-state/scan-state-leaf.type';
import { debounceTime, delay, distinctUntilChanged, filter, fromEvent, skip, tap } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';
import { any, hasValue, isMobile } from '@openmina/shared';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { ScanStateSetActiveLeaf } from '@snarks/scan-state/scan-state.actions';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import {
  selectScanStateHighlightSnarkPool,
  selectScanStateOpenSidePanel,
  selectScanStateSideBarResized,
  selectScanStateTreeView,
} from '@snarks/scan-state/scan-state.state';
import { AppSelectors } from '@app/app.state';


interface SSTree {
  leaf: ScanStateLeaf;
  size?: number;
  children: SSTree[];
}

@Component({
  selector: 'mina-scan-state-tree-chart',
  templateUrl: './scan-state-tree-chart.component.html',
  styleUrls: ['./scan-state-tree-chart.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'fx-col-full-cent w-100' },
})
export class ScanStateTreeChartComponent extends StoreDispatcher implements OnInit, OnChanges {

  @Input({ required: true }) tree: ScanStateTree;
  @Input() activeLeaf: ScanStateLeaf;
  @Input() blockHeight: number;

  @ViewChild('chart', { static: true }) private chartContainer: ElementRef<HTMLDivElement>;
  @ViewChild('tooltip', { static: true }) private tooltipRef: ElementRef<HTMLDivElement>;

  private tooltip: d3.Selection<HTMLDivElement, unknown, HTMLElement, undefined>;

  private width: number;
  private height: number;
  private padding: number = 1;
  private svg: any;
  private leafsHolder: any;
  private jobsHolder: any;
  private initialized: boolean;
  private root: d3.HierarchyNode<any>;
  private cellsData: any;
  private baseJobs: any;
  private highlightRect: any;
  private linesHolder: any;
  private lines: any;
  private numOfRows: number = 8;
  private leafWidth: number;
  private treeView: boolean;
  private highlightSnarks: boolean;

  constructor(private router: Router) { super(); }

  ngOnInit(): void {
    this.width = this.chartContainer.nativeElement.offsetWidth;
    this.height = this.chartContainer.nativeElement.offsetHeight;
    this.tooltip = d3.select(this.tooltipRef.nativeElement);

    this.svg = d3
      .select(this.chartContainer.nativeElement)
      .append('svg')
      .attr('width', this.width)
      .attr('height', this.height)
      .attr('viewBox', [0, 0, this.width, this.height]);

    this.leafsHolder = this.svg
      .append('g');

    this.linesHolder = this.svg
      .append('g');

    this.jobsHolder = this.svg
      .append('g')
      .attr('width', this.width)
      .attr('height', 20)
      .attr('transform', `translate(0, ${this.height - 15})`);

    this.initialized = true;
    this.listenToHighlightSnarkPoolChange();
    this.drawTree();
    this.handleResizing();
    this.listenToTreeViewChange();
  }

  ngOnChanges(): void {
    if (!this.initialized) {
      return;
    }
    this.drawTree();
  }

  private listenToTreeViewChange(): void {
    this.select(selectScanStateTreeView, tv => {
      this.treeView = tv;
      this.updateLeafs(true);
      this.updateLines();
      this.updateHighlightRect(true);
    });
  }

  private handleResizing(): void {
    fromEvent(window, 'resize')
      .pipe(untilDestroyed(this), debounceTime(200))
      .subscribe(() => this.redrawChart());
    this.select(selectScanStateSideBarResized, () => this.redrawChart(), filter(Boolean));
    this.select(selectScanStateOpenSidePanel, () => this.redrawChart(), delay(400), skip(1));
    this.select(AppSelectors.menu, () => this.redrawChart(),
      delay(400),
      distinctUntilChanged(),
      skip(1),
      filter(() => !isMobile()),
    );
  }

  private redrawChart(): void {
    this.width = this.chartContainer.nativeElement.offsetWidth;
    this.height = this.chartContainer.nativeElement.offsetHeight;

    this.svg
      .attr('width', this.width)
      .attr('height', this.height)
      .attr('viewBox', [0, 0, this.width, this.height]);

    this.jobsHolder
      .attr('width', this.width)
      .attr('transform', `translate(0, ${this.height - 15})`);

    this.drawTree(true);
  }

  private drawTree(redraw?: boolean): void {
    const tree = buildTree(this.tree.leafs);
    this.root = d3.hierarchy<any>(tree);
    this.root.sum((d: SSTree) => d.size);
    d3.partition()
      .size([this.width, this.height - 10])(this.root);

    const seventhRowItem = this.root.descendants().filter((node: any) => node.depth === this.numOfRows - 2)[0];
    this.leafWidth = any(seventhRowItem).x1 - any(seventhRowItem).x0;

    this.updateLeafs(redraw);
    this.updateLines();
    this.updateBaseJobs(redraw);
    this.updateHighlightRect(redraw);
  }

  private updateHighlightRect(redraw?: boolean): void {
    if (redraw) {
      this.svg.select('rect.highlight').remove();
    }
    this.highlightRect = this.getHighlightRect();
    const overlayIndex = this.activeLeaf?.jobIndex;
    if (hasValue(overlayIndex) && overlayIndex < this.root.descendants().length) {
      const overlayCell: any = this.root.descendants()[overlayIndex];
      const width = (this.treeView && overlayCell.depth < this.numOfRows - 1) ? this.leafWidth : (overlayCell.x1 - overlayCell.x0 - this.padding);
      this.highlightRect
        .attr('width', width)
        .attr('height', overlayCell.y1 - overlayCell.y0 - this.padding)
        .attr('transform', `translate(${this.treeView ? overlayCell.transform?.x : overlayCell.x0},${this.treeView ? overlayCell.transform?.y : overlayCell.y0})`)
        .style('pointer-events', 'none')
        .style('display', 'block');

    } else {
      this.highlightRect
        .style('display', 'none');
    }
  }

  private getHighlightRect(): any {
    const selection = this.svg.select('rect.highlight');
    if (selection.size() > 0) {
      return selection;
    }
    return this.leafsHolder
      .append('rect')
      .attr('class', 'highlight')
      .style('fill', 'var(--selected-tertiary)')
      .style('stroke', 'var(--selected-primary)')
      .style('stroke-width', 2)
      .style('display', 'none');
  }

  private updateBaseJobs(redraw?: boolean): void {
    if (redraw) {
      this.jobsHolder.selectAll('path').remove();
    }
    const jobs = this.tree.leafs.slice(-128);
    const leafCount = jobs.length;
    const lineWidth = this.width / leafCount;

    this.baseJobs = this.jobsHolder
      .selectAll('path')
      .data(jobs);

    const enteringCells = this.baseJobs
      .enter()
      .append('path')
      .attr('d', (d: any, i: number) => {
        if (d.job) {
          switch (d.job.kind) {
            case 'Payment':
              return ScanStateTreeChartComponent.getCirclePath(i, lineWidth);
            case 'Zkapp':
              return ScanStateTreeChartComponent.getStarPath();
            case 'Coinbase':
              return ScanStateTreeChartComponent.getSquarePath(i, lineWidth);
            case 'FeeTransfer':
              return ScanStateTreeChartComponent.getTrianglePath(i, lineWidth);
            default:
              return ScanStateTreeChartComponent.getLinePath(i, lineWidth);
          }
        }
        return ScanStateTreeChartComponent.getLinePath(i, lineWidth);
      })
      .attr('transform', (d: any, i: number): string | undefined => {
        if (d.job && d.job.kind === 'Zkapp') {
          return `translate(${i * lineWidth + lineWidth / 2},${9})`;
        }
        return undefined;
      })
      .attr('fill', (d: any) => {
        if (d.job) {
          switch (d.job.kind) {
            case 'Coinbase':
              return 'var(--code-teal)';
            case 'FeeTransfer':
              return 'var(--code-purple)';
            case 'Payment':
              return 'var(--base-primary)';
            case 'Zkapp':
              return 'var(--code-magenta)';
            default:
              return 'var(--base-container)';
          }
        }
        return 'var(--base-tertiary)';
      });
    this.baseJobs = this.baseJobs.merge(enteringCells);
    this.baseJobs
      .exit()
      .remove();
  }

  private updateLeafs(redraw?: boolean): void {
    if (redraw) {
      this.leafsHolder.selectAll('rect').remove();
    }

    this.cellsData = this.leafsHolder
      .selectAll('rect')
      .data(this.root.descendants());
    const enteringCells = this.cellsData
      .enter()
      .append('rect')
      .merge(this.cellsData)
      .attr('transform', (d: any) => {
        if (!this.treeView) {
          return `translate(${d.x0},${d.y0})`;
        }
        const rowWidth = (d.depth < this.numOfRows - 2) ? (this.leafWidth / (2 ** (this.numOfRows - d.depth))) : (d.x1 - d.x0);
        const x0 = (d.depth < this.numOfRows - 2) ? ((d.x0 + d.x1) / 2 - rowWidth / 2 - (this.leafWidth / 2)) : d.x0;
        const y0 = d.y0;
        d.transform = { x: x0, y: y0 };
        return `translate(${x0},${y0})`;
      })
      .attr('width', (d: any) => (this.treeView && d.depth < this.numOfRows - 1) ? this.leafWidth : (d.x1 - d.x0))
      .attr('height', (d: any) => d.y1 - d.y0)
      .style('stroke', 'var(--base-background)')
      .style('stroke-width', '1px')
      .style('fill', (d: any) => rectsFill(d, this.highlightSnarks))
      .on('mouseover', (event: MouseEvent & { target: HTMLElement }, d: any) => this.mouseOverHandle(event, d))
      .on('mouseout', (event: MouseEvent & { target: HTMLElement }, d: any) => this.mouseOutHandler(event, d))
      .on('click', (_event: MouseEvent & { target: HTMLElement }, d: any) => {
        this.dispatch(ScanStateSetActiveLeaf, d.data.leaf);
        this.router.navigate([Routes.SNARKS, Routes.SCAN_STATE, this.blockHeight], {
          queryParamsHandling: 'merge',
          queryParams: { jobId: d.data.leaf.bundle_job_id },
        });
      })
      .classed('pulse-effect', (d: any) => this.hasPulseEffect(d));
    this.cellsData = this.cellsData.merge(enteringCells);
    this.cellsData
      .exit()
      .remove();
  }

  private hasPulseEffect(d: any): boolean {
    return this.highlightSnarks && d?.data.leaf.status === ScanStateLeafStatus.Pending && !d.data.leaf.snark;
  }

  private updateLines(): void {
    if (!this.treeView) {
      this.linesHolder.selectAll('line').remove();
      return;
    }
    const linksData = this.root.links().filter((link: any) => link.source.depth <= this.numOfRows - 3 && link.target.depth <= this.numOfRows - 3);

    this.lines = this.linesHolder
      .selectAll('line')
      .data(linksData);

    this.lines.enter() // Update existing lines and enter new lines
      .append('line')
      .merge(this.lines)
      .attr('x1', (d: any) => {
        const offset = this.leafWidth / 2 * ((d.source.x0 + d.source.transform.x > d.target.x0 + d.target.transform.x) ? -1 : 1);
        return (d.source.x0 + d.source.x1) / 2 + offset;
      })
      .attr('x2', (d: any) => (d.target.x0 + d.target.x1) / 2)
      .attr('y1', (d: any) => (d.source.y0 + d.source.y1) / 2)
      .attr('y2', (d: any) => d.target.y0 + 0.5)
      .style('stroke', (d: any) => {
        if (d.source.data.leaf.status === 'Empty' || d.target.data.leaf.status === 'Empty') {
          return 'var(--base-divider)';
        }
        return 'var(--base-tertiary)';
      })
      .style('stroke-width', (d: any) => {
        if (d.source.data.leaf.status === 'Empty' || d.target.data.leaf.status === 'Empty') {
          return '1px';
        }
        return '0.7px';
      });

    this.lines.exit().remove(); // Remove any lines that are no longer needed
  }

  private mouseOverHandle(event: MouseEvent & { target: HTMLElement }, d: any): void {
    const status = d.data.leaf.status === ScanStateLeafStatus.Pending && d.data.leaf.commitment && !d.data.leaf.snark
      ? 'Ongoing' : d.data.leaf.status;
    const selection = this.tooltip.html(`
      <div class="fx-row-vert-cent flex-between h-sm">
         <div>Status</div>
         <div style="color:${ScanStateTreeChartComponent.getColorByStatus(d.data.leaf.status, this.highlightSnarks)}"
              class="f-600">${status}</div>
      </div>
      <div class="fx-row-vert-cent flex-between h-sm">
        <div>Kind</div>
        <div class="f-600 ${ScanStateTreeChartComponent.getClassByKind(d.data.leaf.job?.kind)}">${d.data.leaf.job?.kind || '-'}</div>
      </div>
      <div class="fx-row-vert-cent flex-between h-sm">
        <div>Sequence no.</div>
        <div class="${d.data.leaf.seq_no ? 'primary' : ''}">${d.data.leaf.seq_no || '-'}</div>
      </div>
      <div class="fx-row-vert-cent flex-between h-sm">
        <div>Commitment</div>
        <div class="${d.data.leaf.commitment ? 'primary' : ''}">${!!d.data.leaf.commitment}</div>
      </div>
      <div class="fx-row-vert-cent flex-between h-sm">
        <div>Snark</div>
        <div class="${d.data.leaf.snark ? 'primary' : ''}">${!!d.data.leaf.snark}</div>
      </div>
    `)
      .style('display', 'block');

    d3.select(event.target).style('fill', (d: any) => ScanStateTreeChartComponent.getColorByStatus(d.data.leaf.status, this.highlightSnarks));

    const nodeRect = event.target.getBoundingClientRect();
    const tooltipWidth = selection.node().getBoundingClientRect().width;

    const chartLeft = this.chartContainer.nativeElement.getBoundingClientRect().left;
    let desiredLeft = Math.min(nodeRect.left + nodeRect.width / 2 - tooltipWidth / 2, chartLeft + this.width - tooltipWidth);
    desiredLeft = Math.max(desiredLeft, chartLeft);
    selection
      .style('left', `${desiredLeft}px`)
      .style('top', `${nodeRect.bottom + window.scrollY + 12}px`);
  }

  private mouseOutHandler(event: MouseEvent & { target: HTMLElement }, d: any): void {
    this.tooltip.style('display', 'none');

    d3.select(event.target).style('fill', (d: any) => rectsFill(d, this.highlightSnarks));
  }

  private static getColorByStatus(status: ScanStateLeafStatus, highlight: boolean): string {
    if (status === ScanStateLeafStatus.Done) {
      return 'var(--special-selected-alt-2-primary)';
    } else if (status === ScanStateLeafStatus.Todo) {
      return 'var(--base-primary)';
    } else if (status === ScanStateLeafStatus.Pending) {
      if (highlight) {
        return 'var(--aware-primary)';
      }
      return 'var(--base-primary)';
    }
    return 'var(--base-tertiary)';
  }

  private static getClassByKind(kind: string): string {
    if (kind === 'Coinbase') {
      return 'code-teal';
    } else if (kind === 'FeeTransfer') {
      return 'code-purple';
    } else if (kind === 'Zkapp') {
      return 'code-magenta';
    } else if (kind === 'Payment') {
      return 'primary';
    }
    return 'tertiary';
  }

  private static getStarPath(): string {
    return d3.symbol()
      .type(d3.symbolStar)
      .size(17)();
  }

  private static getCirclePath(i: number, lineWidth: number): string {
    const circleRadius = 3.1;
    const lineHeight = 10;
    const circleCenterX = i * lineWidth + lineWidth / 2;
    const y = 7 + lineHeight / 2 - circleRadius;
    return `M ${circleCenterX},${y} m -${circleRadius},0 a ${circleRadius},${circleRadius} 0 1,0 ${circleRadius * 2},0 a ${circleRadius},${circleRadius} 0 1,0 -${circleRadius * 2},0`;
  }

  private static getSquarePath(i: number, lineWidth: number): string {
    const squareSize = 6;
    const x = i * lineWidth + lineWidth / 2 - squareSize / 2;
    const lineHeight = 18;
    const y = lineHeight / 2 - squareSize / 2;

    return `M${x},${y}h${squareSize}v${squareSize}h-${squareSize}z`;
  }

  private static getTrianglePath(i: number, lineWidth: number): string {
    const containerWidth = lineWidth;
    const triangleSize = 7;
    const maxLineWidth = triangleSize / (Math.sqrt(3) / 2);
    const effectiveLineWidth = Math.min(maxLineWidth, lineWidth);

    const height = (Math.sqrt(3) * effectiveLineWidth) / 2;
    const xOffset = (containerWidth - effectiveLineWidth) / 2;
    const x = i * lineWidth + xOffset;
    const y = 8.5 - height / 2;

    return `M ${x},${y + height} L ${x + effectiveLineWidth / 2},${y} L ${x + effectiveLineWidth},${y + height} Z`;
  }

  private static getLinePath(i: number, lineWidth: number): string {
    const x = i * lineWidth + 0.5;
    const y = 10;
    const width = lineWidth - 1;
    const height = 2;

    return `M ${x} ${y} h ${width} v ${-height} h ${-width} z`;
  }

  private listenToHighlightSnarkPoolChange(): void {
    this.select(selectScanStateHighlightSnarkPool, (highlight: boolean) => {
      this.leafsHolder.selectAll('rect')
        .style('fill', (d: any) => rectsFill(d, highlight))
        .classed('pulse-effect', (d: any) => this.hasPulseEffect(d));
    }, tap(h => this.highlightSnarks = h), skip(1));
  }
}

function buildTree(leafs: ScanStateLeaf[]): SSTree {
  const root: SSTree = { leaf: leafs[0], children: [] };
  const stack: [SSTree, number][] = [[root, 0]];

  while (stack.length > 0) {
    const [currentNode, currentIndex] = stack.pop()!;

    const leftChildIndex = 2 * currentIndex + 1;
    const rightChildIndex = 2 * currentIndex + 2;

    if (leftChildIndex < leafs.length) {
      const leftChild: SSTree = { leaf: leafs[leftChildIndex], children: [] };
      currentNode.children.push(leftChild);
      stack.push([leftChild, leftChildIndex]);
    }

    if (rightChildIndex < leafs.length) {
      const rightChild: SSTree = { leaf: leafs[rightChildIndex], children: [] };
      currentNode.children.push(rightChild);
      stack.push([rightChild, rightChildIndex]);
    }

    if (leftChildIndex >= leafs.length && rightChildIndex >= leafs.length) {
      currentNode.size = 1;
    }
  }

  return root;
}

const rectsFill = (d: any, highlight: boolean): string => {
  if (!d?.data.leaf) {
    return 'var(--base-container)';
  }
  if (d.data.leaf.status === ScanStateLeafStatus.Done) {
    return 'var(--special-selected-alt-2-tertiary)';
  } else if (d.data.leaf.status === ScanStateLeafStatus.Todo) {
    return 'var(--base-tertiary)';
  } else if (d.data.leaf.status === ScanStateLeafStatus.Pending) {
    if (highlight) {
      return 'var(--aware-tertiary)';
    }
    return 'var(--base-tertiary)';
  }
  return 'var(--base-container)';
};
