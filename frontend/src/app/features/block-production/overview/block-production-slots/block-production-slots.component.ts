import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  BlockProductionOverviewService, BlockProductionSlot,
} from '@app/features/block-production/overview/block-production-overview.service';
import * as d3 from 'd3';

@Component({
  selector: 'mina-block-production-slots',
  templateUrl: './block-production-slots.component.html',
  styleUrls: ['./block-production-slots.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100' },
})
export class BlockProductionSlotsComponent extends StoreDispatcher implements AfterViewInit {
  @ViewChild('graph') private graph: ElementRef<HTMLDivElement>;
  @ViewChild('tooltip') private tooltipRef: ElementRef<HTMLDivElement>;
  private tooltip: d3.Selection<HTMLDivElement, unknown, HTMLElement, undefined>;

  // Define margins as a property of the class
  private margin = { top: 0, right: 0, bottom: 0, left: 0 };
  private width: number;
  private height: number;
  private slots: BlockProductionSlot[] = [];

  constructor(private service: BlockProductionOverviewService) { super(); }

  ngAfterViewInit(): void {
    this.service.getSlots().subscribe(slots => {
      this.slots = slots;
      this.initSlots();
    });
  }

  private initSlots(): void {
    this.width = this.graph.nativeElement.clientWidth;
    this.height = this.graph.nativeElement.clientHeight;
    this.tooltip = d3.select(this.tooltipRef.nativeElement);

    const svg = d3.select(this.graph.nativeElement)
      .append('svg')
      .attr('width', this.width + this.margin.left + this.margin.right)
      .attr('height', this.height + this.margin.top + this.margin.bottom);

    const slotsGroup = svg.append('g');
    const slotWidth = 8;
    const slotHeight = 8;
    const slotMargin = 1;
    const slotsPerRow = Math.floor(this.width / (slotWidth + slotMargin));
    const slotGroup = slotsGroup.selectAll('rect')
      .data(this.slots)
      .enter()
      .append('rect')
      .attr('width', slotWidth)
      .attr('height', slotHeight)
      .attr('x', (d, i) => {
        return (i % slotsPerRow) * (slotWidth + slotMargin);
      })
      .attr('y', (d, i) => {
        const rowIndex = Math.floor(i / slotsPerRow);
        return rowIndex * (slotHeight + slotMargin);
      })

      .attr('fill', d => {
        const prefix = 'var(--';
        const suffix = ')';
        let color = 'base-container';
        if (d.finished) {
          color = 'selected-tertiary';
        }

        if (d.canonical) {
          color = 'success-primary';
        } else if (d.orphaned) {
          color = 'special-selected-alt-1-primary';
        } else if (d.missedRights) {
          color = 'warn-primary';
        } else if (d.futureRights) {
          color = 'base-secondary';
        } else if (d.missed) {
          return undefined;
        } else if (!d.finished) {
          color = 'base-container';
        }

        if (d.active) {
          color = 'selected-primary';
        }
        return `${prefix}${color}${suffix}`;
      })
      // add tooltip on hover with color
      .on('mouseover', (event: MouseEvent & {
        target: HTMLElement
      }, d: BlockProductionSlot) => this.mouseOverHandle(event, d))
      .on('mouseout', (event: MouseEvent & {
        target: HTMLElement
      }, d: BlockProductionSlot) => this.mouseOutHandler(event, d));
  }

  private mouseOverHandle(event: MouseEvent & { target: HTMLElement }, d: BlockProductionSlot): void {
    const color = event.target.getAttribute('fill');
    const selection = this.tooltip.html(`
      <div class="fx-row-vert-cent flex-between h-sm">
         <div class="f-600 text-nowrap">${d.height}</div>
      </div>
    `)
      .style('display', 'block');

    const nodeRect = event.target.getBoundingClientRect();
    const tooltipWidth = selection.node().getBoundingClientRect().width;

    const chartLeft = this.graph.nativeElement.getBoundingClientRect().left;
    let desiredLeft = Math.min(nodeRect.left + nodeRect.width / 2 - tooltipWidth / 2, chartLeft + this.width - tooltipWidth);
    desiredLeft = Math.max(desiredLeft, chartLeft);
    selection
      .style('left', `${desiredLeft}px`)
      .style('top', `${nodeRect.bottom + window.scrollY + 12}px`);
  }

  private mouseOutHandler(event: MouseEvent & { target: HTMLElement }, d: BlockProductionSlot): void {
    this.tooltip.style('display', 'none');
  }
}
