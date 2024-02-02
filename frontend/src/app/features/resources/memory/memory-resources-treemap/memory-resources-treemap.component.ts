import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, HostListener, NgZone, ViewChild } from '@angular/core';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import * as d3 from 'd3';
import { hierarchy, HierarchyNode, HierarchyRectangularNode, ScaleLinear, Selection, treemapBinary, treemapDice, treemapSlice, treemapSquarify } from 'd3';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  selectMemoryResources,
  selectMemoryResourcesActiveResource,
  selectMemoryResourcesBreadcrumbs,
  selectMemoryResourcesTreemapView,
} from '@resources/memory/memory-resources.state';
import { debounceTime, delay, distinctUntilChanged, filter, fromEvent, skip, tap } from 'rxjs';
import { ResourcesSizePipe } from '@resources/memory/memory-resources.pipe';
import { MemoryResourcesSetActiveResource } from '@resources/memory/memory-resources.actions';
import { untilDestroyed } from '@ngneat/until-destroy';
import { TreemapView } from '@shared/types/resources/memory/treemap-view.type';
import { isDesktop, isMobile, TooltipService } from '@openmina/shared';
import { selectAppMenu } from '@app/app.state';

@Component({
  selector: 'app-memory-resources-treemap',
  templateUrl: './memory-resources-treemap.component.html',
  styleUrls: ['./memory-resources-treemap.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MemoryResourcesTreemapComponent extends StoreDispatcher implements AfterViewInit {

  private breadcrumbs: MemoryResource[] = [];
  private activeResource: MemoryResource;
  private isFirstTime: boolean = true;
  private lastClicked: MemoryResource;
  private baseResource: MemoryResource;
  private treemapView: TreemapView;
  private tooltip: HTMLDivElement;

  @ViewChild('treeMapChart') private treeMapRef: ElementRef<HTMLDivElement>;

  private width: number;
  private height: number;
  private svg: Selection<SVGSVGElement, MemoryResource | unknown, null, undefined>;
  private group: Selection<SVGGElement, MemoryResource | unknown, SVGSVGElement, MemoryResource>;
  private treemap: HierarchyRectangularNode<MemoryResource>;
  private xScale: ScaleLinear<number, number>;
  private yScale: ScaleLinear<number, number>;

  @HostListener('document:keydown.escape')
  onKeydownHandler(): void {
    if (this.breadcrumbs.length > 1) {
      const node = this.findNodeByName(this.breadcrumbs[this.breadcrumbs.length - 1], this.treemap);
      this.zoomOut(node);
    }
  }

  constructor(private ngZone: NgZone,
              private sizePipe: ResourcesSizePipe,
              private tooltipService: TooltipService) { super(); }

  ngAfterViewInit(): void {
    this.tooltip = this.tooltipService.graphTooltip;
    this.tooltip.style.display = 'none';

    this.listenToResizeEvent();
    this.listenToMemoryResourcesChanges();
  }

  private listenToResizeEvent(): void {
    this.ngZone.runOutsideAngular(() => {
      fromEvent(window, 'resize')
        .pipe(untilDestroyed(this), debounceTime(200))
        .subscribe(() => this.redrawChart());
      this.select(
        selectAppMenu,
        () => this.redrawChart(),
        delay(350),
        distinctUntilChanged(),
        skip(1),
        filter(() => isDesktop()),
      );
    });
  }

  private listenToMemoryResourcesChanges(): void {
    this.ngZone.runOutsideAngular(() => {
      this.select(selectMemoryResourcesTreemapView, (treemapView: TreemapView) => {
        if (!this.treemapView) {
          this.treemapView = treemapView;
        } else {
          this.treemapView = treemapView;
          this.createTreemapValues(this.baseResource);
          const node = this.findNodeByName(this.activeResource, this.treemap);
          this.zoomIn(node, false);
        }
      });
      this.select(
        selectMemoryResources,
        (resource: MemoryResource) => {
          this.baseResource = resource;
          this.createChart(resource);
        },
        tap(r => {
          if (!r) {
            this.svg?.remove();
            this.svg = undefined;
            this.hideTooltip();
            this.isFirstTime = true;
            this.baseResource = undefined;
            this.lastClicked = undefined;
          }
          return r;
        }),
        filter(Boolean),
      );

      this.select(selectMemoryResourcesActiveResource, (resource: MemoryResource) => {
        this.activeResource = resource;
        if (!this.isFirstTime) {
          if (this.lastClicked !== resource) {
            this.lastClicked = resource;
            const node = this.findNodeByName(resource, this.treemap);
            this.zoomIn(node, false);
          }
        }
        this.isFirstTime = false;
      }, filter(Boolean));

      this.select(selectMemoryResourcesBreadcrumbs, (breadcrumbs: MemoryResource[]) => {
        this.breadcrumbs = breadcrumbs;
      }, filter(Boolean));
    });
  }

  private redrawChart(): void {
    this.bindWidthAndHeight();
    this.svg
      .attr('width', this.width)
      .attr('height', this.height);
    this.createTreemapValues(this.baseResource);
    this.bindAxisScales();
    const node = this.findNodeByName(this.activeResource, this.treemap);
    this.zoomIn(node);
  }

  private createSVG(): void {
    if (this.svg) {
      return;
    }
    this.svg = d3.select(this.treeMapRef.nativeElement)
      .append('svg')
      .attr('width', this.width)
      .attr('height', this.height)
      .style('font-size', '10px');
  }

  private createChart(resource: MemoryResource): void {
    this.bindWidthAndHeight();
    this.createSVG();
    this.createTreemapValues(resource);
    this.bindAxisScales();
    this.createTreemapGElement();

    this.render(this.group, this.treemap);
    this.position(this.group);
    this.fadeTexts(this.group);
  }

  private createTreemapGElement(): void {
    const existingGroup = this.svg.select<SVGGElement>('.treemap');

    if (existingGroup.empty()) {
      this.group = this.svg
        .append<SVGGElement>('g')
        .attr('class', 'treemap');
    } else {
      this.group = existingGroup;
    }
  }

  private bindAxisScales(): void {
    this.xScale = d3.scaleLinear().rangeRound([0, this.width]);
    this.yScale = d3.scaleLinear().rangeRound([0, this.height]);
  }

  private createTreemapValues(resource: MemoryResource): void {
    const tile = (node: HierarchyRectangularNode<MemoryResource>, x0: number, y0: number, x1: number, y1: number) => {
      switch (this.treemapView) {
        case TreemapView.BINARY:
          treemapBinary(node, 0, 0, this.width, this.height);
          break;
        case TreemapView.SQUARE:
          treemapSquarify(node, 0, 0, this.width, this.height);
          break;
        case TreemapView.SLICE:
          treemapSlice(node, 0, 0, this.width, this.height);
          break;
        case TreemapView.DICE:
          treemapDice(node, 0, 0, this.width, this.height);
          break;
        default:
          treemapBinary(node, 0, 0, this.width, this.height);
      }
      for (const child of node.children) {
        child.x0 = x0 + (child.x0 / this.width) * (x1 - x0);
        child.x1 = x0 + (child.x1 / this.width) * (x1 - x0);
        child.y0 = y0 + (child.y0 / this.height) * (y1 - y0);
        child.y1 = y0 + (child.y1 / this.height) * (y1 - y0);
      }
    };

    const root = hierarchy<MemoryResource>(resource)
      .sum((d: MemoryResource) => d.children.length ? 0 : (d.value || 0.01))
      .sort((a: HierarchyNode<MemoryResource>, b: HierarchyNode<MemoryResource>) => b.value - a.value);
    this.treemap = d3.treemap<MemoryResource>().tile(tile)(root);
  }

  private bindWidthAndHeight(): void {
    this.width = this.treeMapRef.nativeElement.offsetWidth - 2;
    this.height = this.treeMapRef.nativeElement.offsetHeight - 2;
  }

  private render(group: Selection<SVGGElement, MemoryResource | unknown, SVGSVGElement, null>, root: HierarchyRectangularNode<MemoryResource>): void {
    const node = group
      .selectAll<SVGGElement, HierarchyRectangularNode<MemoryResource>>('g.node')
      .data(root.children)
      .join('g')
      .attr('class', 'node');

    node
      .attr('cursor', (d: HierarchyRectangularNode<MemoryResource>) => d.children ? 'pointer' : null)
      .on('click', (_, d: HierarchyRectangularNode<MemoryResource>) => d.children ? this.zoomIn(d) : null)
      .on('mouseenter', (event: { target: SVGGElement }, d: HierarchyRectangularNode<MemoryResource>) => {
        if (d.data.children.length) {
          const g = d3.select(event.target);
          g.raise();
          g.select('rect')
            .attr('fill', 'var(--base-tertiary2)')
            .attr('stroke', 'var(--base-tertiary)')
            .attr('stroke-width', 2);
          g.select('text.name')
            .attr('fill', 'var(--base-primary)');
          g.select('text.val')
            .attr('fill', 'var(--base-secondary)');
          g.select('text.child-count')
            .attr('fill', 'var(--base-primary)');
          g.select('rect.child-bg')
            .attr('fill', 'var(--base-container)');
        }
        this.showTooltip(event, d);
      })
      .on('mousemove', (event: { target: SVGGElement }) => {
        this.updateTooltipPosition(event);
      })
      .on('mouseleave', (event: { target: SVGGElement }, d: HierarchyRectangularNode<MemoryResource>) => {
        if (d.data.children.length) {
          const g = d3.select(event.target);
          g.select('rect')
            .attr('fill', 'var(--base-surface-top)')
            .attr('stroke', 'var(--base-tertiary)')
            .attr('stroke-width', 0.5);
          g.select('text.name')
            .attr('fill', 'var(--base-primary)');
          g.select('text.val')
            .attr('fill', 'var(--base-tertiary)');
          g.select('text.child-count')
            .attr('fill', 'var(--base-primary)');
          g.select('rect.child-bg')
            .attr('fill', 'var(--base-background)');
        }
        this.hideTooltip();
      });

    if (node.select('rect').empty()) {
      node.append('rect')
        .attr('fill', (d: HierarchyRectangularNode<MemoryResource>) => d.data.children.length ? 'var(--base-surface-top)' : 'var(--base-background)')
        .attr('stroke', 'var(--base-tertiary)')
        .attr('stroke-width', 0.5);
    }

    if (node.select<SVGTextElement>('text.name').empty()) {
      node.append('text')
        .attr('class', 'name')
        .attr('x', 8)
        .attr('y', 16)
        .attr('fill', 'var(--base-primary)')
        .text((d: HierarchyRectangularNode<MemoryResource>) => d.data.name.executableName);
    }

    if (node.select('text.val').empty()) {
      node.append('text')
        .attr('class', 'val')
        .attr('x', 8)
        .attr('y', 28)
        .attr('fill', 'var(--base-tertiary)')
        .text((d: HierarchyRectangularNode<MemoryResource>) => this.sizePipe.transform(d.data.value));
    }

    const rectPadding = 4;
    const marginLeft = 4;
    const nameEl = node.select<SVGTextElement>('text.name');
    const nameElWidth = (nameEl.node()?.getBBox().width + 10) || 0;

    const countText = node.append('text')
      .attr('class', 'child-count')
      .attr('x', nameElWidth + marginLeft + rectPadding)
      .attr('y', 16)
      .attr('fill', 'var(--base-primary)')
      .text((d: HierarchyRectangularNode<MemoryResource>) => d.data.children.length);

    node.append('rect')
      .attr('class', 'child-bg')
      .attr('x', marginLeft + nameElWidth)
      .attr('y', 4)
      .attr('width', () => (countText.node()?.getBBox().width || 0) + (rectPadding * 2))
      .attr('height', 16)
      .attr('rx', 2)
      .attr('fill', 'var(--base-container)');

    node.append('path')
      .attr('class', 'chevron')
      .attr('fill', 'none')
      .attr('stroke', 'var(--base-primary)')
      .attr('stroke-width', '1');

    countText.raise();
  }

  private showTooltip(event: { target: SVGGElement }, d: HierarchyRectangularNode<MemoryResource>) {
    this.tooltip.innerHTML = `
      ${d.data.name.executableName} <span class="tertiary">${this.sizePipe.transform(d.data.value)}</span><br>
        ${d.data.children.length} <span class="tertiary">${d.data.children.length === 1 ? 'child' : 'children'}</span>
  `;
    this.tooltip.style.display = 'block';
    this.updateTooltipPosition(event);
  }

  private updateTooltipPosition(event: any): void {
    const tooltipWidth = this.tooltip.offsetWidth;
    const xMargin = 14;
    const yMargin = 22;

    const x = event.clientX - tooltipWidth / 2 + xMargin;
    const y = event.clientY + yMargin;

    const maxX = window.innerWidth - tooltipWidth - xMargin;
    if (x > maxX) {
      this.tooltip.style.left = maxX + 'px';
    } else {
      this.tooltip.style.left = x + 'px';
    }

    this.tooltip.style.top = y + 'px';
  }

  private hideTooltip(): void {
    this.tooltip.style.display = 'none';
  }

  private position(group: Selection<SVGGElement, MemoryResource | unknown, SVGSVGElement, MemoryResource>): void {
    group
      .selectAll<SVGGElement, HierarchyRectangularNode<MemoryResource>>('g')
      .attr('transform', (d: HierarchyRectangularNode<MemoryResource>) => `translate(${this.xScale(d.x0)},${this.yScale(d.y0)})`)
      .select('rect')
      .attr('width', (d: HierarchyRectangularNode<MemoryResource>) => this.xScale(d.x1) - this.xScale(d.x0))
      .attr('height', (d: HierarchyRectangularNode<MemoryResource>) => this.yScale(d.y1) - this.yScale(d.y0));
  };

  private fadeTexts(group: Selection<SVGGElement, MemoryResource | unknown, SVGSVGElement, MemoryResource>): void {
    group
      .selectAll<SVGGElement, HierarchyRectangularNode<MemoryResource>>('g')
      .nodes()
      .forEach((node: SVGGElement) => {
        const d3node = d3.select<SVGGElement, HierarchyRectangularNode<MemoryResource>>(node);
        const rect = d3node.select<SVGRectElement>('rect:not(.child-bg)');
        const rectWidth = rect?.node()?.getBBox().width || 0;
        const rectHeight = rect?.node()?.getBBox().height || 0;
        const textElement = d3node.select<SVGTextElement>('text.name');
        const textWidth = (textElement?.node()?.getBBox().width || 0) + 10;
        const textNodes = d3.select(node).selectAll('text');
        const childBgRect = d3node.select<SVGRectElement>('rect.child-bg');
        const childBgRectWidth = childBgRect?.node()?.getBBox().width || 0;
        const childCountText = d3node.select<SVGTextElement>('text.child-count');
        const zeroChildren = d3node.datum().data.children.length === 0;
        const requiredWidth = zeroChildren ? textWidth : (textWidth + childBgRectWidth + 20);
        const chevron = d3node.select<SVGRectElement>('path.chevron');
        if (requiredWidth > rectWidth || rectHeight < 30) {
          textNodes.classed('see', false);
          childBgRect.classed('see', false);
        } else {
          textNodes.classed('see', true);
          childCountText.classed('see', !zeroChildren);
          childBgRect.classed('see', !zeroChildren);
          chevron.classed('see', !zeroChildren);
        }

        childCountText.attr('x', textWidth + 4 + 4);
        childBgRect.attr('x', textWidth + 4);
        chevron.attr('d', () => `m${rectWidth - 12} 8 4 4-4 4`);
      });
  }

  private zoomIn(d: HierarchyRectangularNode<MemoryResource>, dispatch: boolean = true): void {
    this.lastClicked = d.data;

    this.xScale.domain([d.x0, d.x1]);
    this.yScale.domain([d.y0, d.y1]);
    const oldG = this.group.attr('pointer-events', 'none');
    const newG = this.svg.append<SVGGElement>('g').attr('class', 'treemap');
    this.render(newG, d);
    this.position(newG);

    this.position(
      oldG.transition().duration(500).remove() as any,
    );

    this.position(
      (newG
        .attr('transform', `scale(2)`)
        .transition().duration(500) as any)
        .on('end', () => this.fadeTexts(newG))
        .attr('transform', `scale(1)`)
        .attrTween('opacity', () => d3.interpolate(0, 1)),
    );

    this.group = newG;

    if (dispatch) {
      this.ngZone.run(() => this.dispatch(MemoryResourcesSetActiveResource, d.data));
    }
  };

  private zoomOut(d: HierarchyRectangularNode<MemoryResource>): void {
    this.lastClicked = d.parent.data;
    const oldG = this.group.attr('pointer-events', 'none');
    const newG = this.svg.insert<SVGGElement>('g', '*').attr('class', 'treemap');

    this.xScale.domain([d.parent.x0, d.parent.x1]);
    this.yScale.domain([d.parent.y0, d.parent.y1]);
    this.render(newG, d.parent);
    this.position(newG);

    this.position(
      newG.style('opacity', 0.6)
        .attr('transform', `scale(2)`)
        .transition().duration(500)
        .on('end', () => this.fadeTexts(newG))
        .style('opacity', 1)
        .attr('transform', `scale(1)`) as any,
    );

    this.position(
      oldG.transition().duration(500)
        .style('opacity', 0).on('end', () => oldG.remove()) as any,
    );

    this.group = newG;
    this.ngZone.run(() => this.dispatch(MemoryResourcesSetActiveResource, d.parent.data));
  };

  private findNodeByName(node: MemoryResource, root: HierarchyRectangularNode<MemoryResource>): HierarchyRectangularNode<MemoryResource> {
    if (
      root.data.name.executableName === node.name.executableName
      && root.data.name.functionName === node.name.functionName
      && root.data.value === node.value
      && root.data.children.length === node.children.length
    ) {
      return root;
    } else if (root.children) {
      let found: HierarchyRectangularNode<MemoryResource> = null;
      root.children.forEach(item => {
        if (!found) {
          found = this.findNodeByName(node, item);
        }
      });

      return found;
    }

    return null;
  }
}
