import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, NgZone, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import * as d3 from 'd3';
import { BlockProductionSlot } from '@shared/types/block-production/overview/block-production-overview-slot.type';
import {
  selectBlockProductionOverviewActiveEpoch,
  selectBlockProductionOverviewFilters,
} from '@block-production/overview/block-production-overview.state';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { debounceTime, filter, fromEvent, skip, tap } from 'rxjs';
import {
  BlockProductionOverviewFilters,
} from '@shared/types/block-production/overview/block-production-overview-filters.type';
import { untilDestroyed } from '@ngneat/until-destroy';
import { ONE_THOUSAND, toReadableDate } from '@openmina/shared';

type MouseEventWithTarget = MouseEvent & {
  target: HTMLElement
};

@Component({
  selector: 'mina-block-production-overview-slots',
  templateUrl: './block-production-overview-slots.component.html',
  styleUrls: ['./block-production-overview-slots.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column flex-grow pb-5 pl-12 p-relative' },
})
export class BlockProductionOverviewSlotsComponent extends StoreDispatcher implements AfterViewInit {
  @ViewChild('graph') private svgHolder: ElementRef<HTMLDivElement>;
  @ViewChild('tooltip') private tooltipRef: ElementRef<HTMLDivElement>;

  slots: BlockProductionSlot[];

  private tooltip: d3.Selection<HTMLDivElement, unknown, HTMLElement, undefined>;
  private svg: d3.Selection<SVGSVGElement, unknown, HTMLElement, undefined>;
  private slotsGroup: d3.Selection<SVGGElement, unknown, HTMLElement, undefined>;
  private slotWidth: number = 8;
  private slotHeight: number = 8;
  private slotMargin: number = 1;
  private width: number;
  private filters: BlockProductionOverviewFilters;
  private hideTooltipTimeout: any;

  constructor(private zone: NgZone) { super(); }

  ngAfterViewInit(): void {
    this.zone.runOutsideAngular(() => {
      this.addSvgWizard();
      this.listenToFilters();
      this.listenToEpochs();
      this.listenToElWidthChange();
    });
  }

  private listenToEpochs(): void {
    this.select(selectBlockProductionOverviewActiveEpoch, (epoch: BlockProductionOverviewEpoch) => {
      this.slots = epoch.slots;

      const slotsRectExist = !this.slotsGroup
        .selectAll<SVGRectElement, BlockProductionSlot>('rect')
        .empty();
      if (slotsRectExist) {
        if (this.slots?.length) {
          this.changeSlotsColors();
        } else {
          this.slotsGroup.selectAll<SVGRectElement, BlockProductionSlot>('rect').remove();
        }
      } else {
        this.initSlots();
      }
      this.detect();
    }, skip(1), filter(epoch => !!epoch?.slots));
  }

  private listenToFilters(): void {
    this.select(selectBlockProductionOverviewFilters, (filters: BlockProductionOverviewFilters) => {
      this.filters = filters;
      if (this.slots?.length) {
        this.changeSlotsColors();
      }
    });
  }

  private listenToElWidthChange(): void {
    this.width = this.svgHolder.nativeElement.clientWidth;
    const widthChange = 'width-change';
    const resizeObserver = new ResizeObserver((entries: ResizeObserverEntry[]) => {
      for (const entry of entries) {
        const widthChangeEvent = new CustomEvent(widthChange);
        entry.target.dispatchEvent(widthChangeEvent);
      }
    });
    resizeObserver.observe(this.svgHolder.nativeElement);

    fromEvent(this.svgHolder.nativeElement, widthChange).pipe(
      tap(() => this.handleElementResizing()),
      debounceTime(400),
      filter(() => this.width !== this.svgHolder.nativeElement.clientWidth),
      tap(() => this.handleElementResizing()),
      untilDestroyed(this),
    ).subscribe();
  }

  private handleElementResizing(): void {
    this.width = this.svgHolder.nativeElement.clientWidth;
    this.svg.attr('width', this.width);
    const slotsPerRow = this.slotsPerRow;
    this.slotsGroup
      ?.selectAll<SVGRectElement, BlockProductionSlot>('rect')
      .attr('x', this.setX(slotsPerRow))
      .attr('y', this.setY(slotsPerRow));
    this.svg.attr('height', this.slotsGroup.node().getBBox().height);
  }

  private addSvgWizard(): void {
    this.width = this.svgHolder.nativeElement.clientWidth;
    this.tooltip = d3.select(this.tooltipRef.nativeElement);

    this.svg = d3.select(this.svgHolder.nativeElement)
      .append('svg')
      .attr('width', this.width);

    this.slotsGroup = this.svg.append('g');
  }

  private initSlots(): void {
    const slotsPerRow = this.slotsPerRow;
    this.slotsGroup.selectAll('rect')
      .data(this.slots)
      .enter()
      .append('rect')
      .attr('x', this.setX(slotsPerRow))
      .attr('y', this.setY(slotsPerRow))
      .attr('fill', (d: BlockProductionSlot) => this.getSlotColor(d))
      .on('mouseover', (event: MouseEventWithTarget, d: BlockProductionSlot) => this.mouseOverHandle(event, d))
      .on('mouseout', (event: MouseEventWithTarget) => {
        d3.select(event.target).attr('stroke', undefined);
        this.hideTooltipTimeout = setTimeout(() => this.tooltip.style('display', 'none'), 100);
      });

    // after everything is added, bind the new height to the svg
    this.svg.attr('height', this.slotsGroup.node().getBBox().height);
  }

  private setY(slotsPerRow: number): (d: BlockProductionSlot, i: number) => number {
    return (_: BlockProductionSlot, i: number) => Math.floor(i / slotsPerRow) * (this.slotHeight + this.slotMargin);
  }

  private setX(slotsPerRow: number): (d: BlockProductionSlot, i: number) => number {
    return (_: BlockProductionSlot, i: number) => (i % slotsPerRow) * (this.slotWidth + this.slotMargin);
  }

  private get slotsPerRow(): number {
    return Math.floor(this.width / (this.slotWidth + this.slotMargin));
  }

  private mouseOverHandle(event: MouseEventWithTarget, d: BlockProductionSlot): void {
    clearTimeout(this.hideTooltipTimeout);
    d3.select(event.target).attr('stroke', 'var(--selected-primary)');
    const selection = this.tooltip.html(`
      <div class="tertiary">Slot ${d.globalSlot}, ${toReadableDate(d.time * ONE_THOUSAND, 'dd MMM, HH:mm')}</div>
      <div class="primary">${this.getSlotText(d)}</div>
    `)
      .style('display', 'flex');

    const nodeRect = event.target.getBoundingClientRect();
    const tooltipWidth = selection.node().getBoundingClientRect().width;

    const chartLeft = this.svgHolder.nativeElement.getBoundingClientRect().left;
    let desiredLeft = Math.min(nodeRect.left + nodeRect.width / 2 - tooltipWidth / 2, chartLeft + this.width - tooltipWidth);
    desiredLeft = Math.max(desiredLeft, chartLeft);
    selection
      .style('left', `${desiredLeft}px`)
      .style('top', `${nodeRect.bottom + window.scrollY + 12}px`);
  }

  private changeSlotsColors(): void {
    this.slotsGroup
      ?.selectAll<SVGRectElement, BlockProductionSlot>('rect')
      .data(this.slots)
      .attr('fill', (d: BlockProductionSlot) => this.getSlotColor(d));
  }

  private getSlotColor(d: BlockProductionSlot): string {
    const prefix = 'var(--';
    const suffix = ')';
    let color = 'base-container';
    if (d.finished) {
      color = 'selected-tertiary';
    }

    if (d.canonical) {
      color = 'success-' + (this.filters?.canonical ? 'primary' : 'tertiary');
    } else if (d.orphaned) {
      color = 'special-selected-alt-1-' + (this.filters?.orphaned ? 'primary' : 'tertiary');
    } else if (d.missed) {
      color = 'warn-' + (this.filters?.missed ? 'primary' : 'tertiary');
    } else if (d.futureRights) {
      color = 'base-' + (this.filters?.future ? 'secondary' : 'divider');
    } else if (!d.finished) {
      color = 'base-container';
    }

    if (d.active) {
      color = 'selected-primary';
    }
    return `${prefix}${color}${suffix}`;
  }

  private getSlotText(d: BlockProductionSlot): string {
    let text = '';
    if (d.finished) {
      text = 'Slot not won';
    }

    if (d.canonical) {
      text = 'Canonical Block Produced';
    } else if (d.orphaned) {
      text = 'Orphaned Block Produced';
    } else if (d.missed) {
      text = 'Missed Block';
    } else if (d.futureRights) {
      text = 'Right to produce a block';
    } else if (!d.finished) {
      text = 'Empty Slot';
    }

    if (d.active) {
      text = 'Currently producing';
    }
    return text;
  }
}
