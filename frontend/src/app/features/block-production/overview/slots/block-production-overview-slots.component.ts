import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, NgZone, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import * as d3 from 'd3';
import {
  BlockProductionOverviewSlot,
} from '@shared/types/block-production/overview/block-production-overview-slot.type';
import { BlockProductionOverviewSelectors } from '@block-production/overview/block-production-overview.state';
import {
  BlockProductionOverviewEpoch,
} from '@shared/types/block-production/overview/block-production-overview-epoch.type';
import { debounceTime, filter, fromEvent, tap } from 'rxjs';
import {
  BlockProductionOverviewFilters,
} from '@shared/types/block-production/overview/block-production-overview-filters.type';
import { untilDestroyed } from '@ngneat/until-destroy';
import { ONE_THOUSAND, toReadableDate } from '@openmina/shared';
import { BlockProductionOverviewActions } from '@block-production/overview/block-production-overview.actions';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';

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

  slots: BlockProductionOverviewSlot[];

  private activeEpoch: BlockProductionOverviewEpoch;
  private activeSlot: BlockProductionOverviewSlot;

  private tooltip: d3.Selection<HTMLDivElement, unknown, HTMLElement, undefined>;
  private svg: d3.Selection<SVGSVGElement, unknown, HTMLElement, undefined>;
  private slotsGroup: d3.Selection<SVGGElement, unknown, HTMLElement, undefined>;
  private slotWidth: number = 8;
  private slotHeight: number = 8;
  private slotMargin: number = 1;
  private width: number;
  private filters: BlockProductionOverviewFilters;
  private hideTooltipTimeout: any;

  constructor(private zone: NgZone,
              private router: Router) { super(); }

  ngAfterViewInit(): void {
    this.zone.runOutsideAngular(() => {
      this.addSvgWizard();
      this.listenToFilters();
      this.listenToEpochs();
      this.listenToActiveSlot();
      this.listenToElWidthChange();
    });
  }

  private listenToEpochs(): void {
    this.select(BlockProductionOverviewSelectors.activeEpoch, (epoch: BlockProductionOverviewEpoch) => {
      this.slots = epoch.slots;

      const slotsRectExist = !this.slotsGroup
        .selectAll<SVGRectElement, BlockProductionOverviewSlot>('rect')
        .empty();
      if (slotsRectExist) {
        if (this.slots?.length) {
          this.changeSlotsColors();
        } else {
          this.slotsGroup.selectAll<SVGRectElement, BlockProductionOverviewSlot>('rect').remove();
        }
      } else {
        this.initSlots();
      }
      this.detect();
    }, filter(epoch => !!epoch?.slots));
  }

  private listenToActiveSlot(): void {
    this.select(BlockProductionOverviewSelectors.activeSlot, (slot: BlockProductionOverviewSlot) => {
      this.activeSlot = slot;
      this.slotsGroup
        .selectAll<SVGRectElement, BlockProductionOverviewSlot>('rect')
        .filter((d: BlockProductionOverviewSlot) => d.slot === this.activeSlot?.slot)
        .attr('stroke', 'var(--selected-primary)');
    });
    this.select(BlockProductionOverviewSelectors.activeEpoch, (epoch: BlockProductionOverviewEpoch) => {
      this.activeEpoch = epoch;
    });
  }

  private listenToFilters(): void {
    this.select(BlockProductionOverviewSelectors.filters, (filters: BlockProductionOverviewFilters) => {
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
      ?.selectAll<SVGRectElement, BlockProductionOverviewSlot>('rect')
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
      .attr('fill', (d: BlockProductionOverviewSlot) => BlockProductionOverviewSlotsComponent.getSlotColor(d, this.filters))
      .on('click', (_: MouseEventWithTarget, d: BlockProductionOverviewSlot) => this.onSlotClick(d))
      .on('mouseover', (event: MouseEventWithTarget, d: BlockProductionOverviewSlot) => this.mouseOverHandle(event, d))
      .on('mouseout', (event: MouseEventWithTarget, d: BlockProductionOverviewSlot) => {
        if (d.slot !== this.activeSlot?.slot) {
          d3.select(event.target).attr('stroke', undefined);
        }
        this.hideTooltipTimeout = setTimeout(() => this.tooltip.style('display', 'none'), 100);
      });

    // after everything is added, bind the new height to the svg
    this.svg.attr('height', this.slotsGroup.node().getBBox().height);
  }

  private setY(slotsPerRow: number): (d: BlockProductionOverviewSlot, i: number) => number {
    return (_: BlockProductionOverviewSlot, i: number) => Math.floor(i / slotsPerRow) * (this.slotHeight + this.slotMargin);
  }

  private setX(slotsPerRow: number): (d: BlockProductionOverviewSlot, i: number) => number {
    return (_: BlockProductionOverviewSlot, i: number) => (i % slotsPerRow) * (this.slotWidth + this.slotMargin);
  }

  private get slotsPerRow(): number {
    return Math.floor(this.width / (this.slotWidth + this.slotMargin));
  }

  private onSlotClick(slot: BlockProductionOverviewSlot): void {
    this.zone.run(() => {
      this.slotsGroup
        .selectAll<SVGRectElement, BlockProductionOverviewSlot>('rect')
        .filter((d: BlockProductionOverviewSlot) => d.slot === this.activeSlot?.slot)
        .attr('stroke', undefined);

      this.dispatch2(BlockProductionOverviewActions.setActiveSlot({ slot: slot.slot }));
      this.router.navigate([Routes.BLOCK_PRODUCTION, Routes.OVERVIEW, this.activeEpoch.epochNumber, slot.slot], { queryParamsHandling: 'merge' });
    });
  }

  private mouseOverHandle(event: MouseEventWithTarget, d: BlockProductionOverviewSlot): void {
    clearTimeout(this.hideTooltipTimeout);
    d3.select(event.target).attr('stroke', 'var(--selected-primary)');
    const selection = this.tooltip.html(`
      <div class="tertiary">Slot ${d.globalSlot}, ${toReadableDate(d.time * ONE_THOUSAND, 'dd MMM, HH:mm')}</div>
      <div class="primary">${BlockProductionOverviewSlotsComponent.getSlotText(d)}</div>
    `)
      .style('display', 'flex');

    const nodeRect = event.target.getBoundingClientRect();
    const tooltipBoundingClientRect = selection.node().getBoundingClientRect();
    const tooltipWidth = tooltipBoundingClientRect.width;
    const tooltipHeight = tooltipBoundingClientRect.height;

    const svgHolderBoundingClientRect = this.svgHolder.nativeElement.getBoundingClientRect();
    const chartLeft = svgHolderBoundingClientRect.left;
    const chartBottom = svgHolderBoundingClientRect.bottom;

    let desiredLeft = Math.min(nodeRect.left + nodeRect.width / 2 - tooltipWidth / 2, chartLeft + this.width - tooltipWidth);
    desiredLeft = Math.max(desiredLeft, chartLeft);
    let desiredTop = nodeRect.bottom + window.scrollY + 12;
    if (desiredTop + tooltipHeight > chartBottom) {
      desiredTop = nodeRect.top - tooltipHeight - 12;
    }

    selection
      .style('left', `${desiredLeft}px`)
      .style('top', `${desiredTop}px`);
  }

  private changeSlotsColors(): void {
    this.slotsGroup
      ?.selectAll<SVGRectElement, BlockProductionOverviewSlot>('rect')
      .data(this.slots)
      .attr('fill', (d: BlockProductionOverviewSlot) => BlockProductionOverviewSlotsComponent.getSlotColor(d, this.filters));
  }

  public static getSlotColor(d: BlockProductionOverviewSlot, filters?: BlockProductionOverviewFilters): string {
    const prefix = 'var(--';
    const suffix = ')';
    let color = 'base-container';
    if (d.finished) {
      color = 'selected-tertiary';
    }

    if (d.canonical) {
      color = 'success-' + (filters?.canonical ? 'primary' : 'tertiary');
    } else if (d.orphaned) {
      color = 'special-selected-alt-1-' + (filters?.orphaned ? 'primary' : 'tertiary');
    } else if (d.missed) {
      color = 'warn-' + (filters?.missed ? 'primary' : 'tertiary');
    } else if (d.futureRights) {
      color = 'base-' + (filters?.future ? 'secondary' : 'divider');
    } else if (!d.finished) {
      color = 'base-container';
    }

    if (d.active) {
      color = 'selected-primary';
    }
    return `${prefix}${color}${suffix}`;
  }

  public static getSlotText(d: BlockProductionOverviewSlot): string {
    let text = '';
    if (d.finished) {
      text = 'Block by another producer';
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
