import { ChangeDetectionStrategy, Component, ComponentRef, EventEmitter, Input } from '@angular/core';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { ComponentPortal } from '@angular/cdk/portal';
import { take } from 'rxjs';
import { FlameTimeGraphTooltipComponent } from '../flame-time-graph-tooltip/flame-time-graph-tooltip.component';
import { OpenminaSharedModule } from '../../../openmina-shared.module';
import { ManualDetection } from '../../../base-classes/manual-detection.class';
import { CommonModule } from '@angular/common';
import { REQUIRED } from '../../../constants/angular';

export interface FlameTimeGraph {
  title: string;
  totalTime: number;
  totalCount: number;
  columns: FlameTimeGraphColumn[];
}

export interface FlameTimeGraphColumn {
  count: number;
  meanTime: number;
  maxTime: number;
  totalTime: number;
  squareCount: number;
}

const Y_STEPS = ['1000s', '100s', '10s', '1s', '100ms', '10ms', '1ms', '100μs', '10μs'];
const X_STEPS = ['1μs', '10μs', '100μs', '1ms', '10ms', '100ms', '1s', '10s', '100s'];
const RANGES = ['1μs - 10μs', '10μs - 100μs', '100μs - 1ms', '1ms - 10ms', '10ms - 100ms', '100ms - 1s', '1s - 10s', '10s - 100s', '> 100s'];
const trackSteps = (index: number) => index;

@Component({
    imports: [OpenminaSharedModule, CommonModule],
    selector: 'mina-flame-time-graph',
    templateUrl: './flame-time-graph.component.html',
    styleUrls: ['./flame-time-graph.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class FlameTimeGraphComponent extends ManualDetection {

  @Input(REQUIRED) checkpoint: FlameTimeGraph;
  @Input(REQUIRED) condensedView: boolean;
  @Input() xSteps: string[] = X_STEPS;
  @Input() ySteps: string[] = Y_STEPS;
  @Input() ranges: string[] = RANGES;
  @Input() visible: boolean = true;
  @Input() menuCollapsed: boolean;

  readonly onClickOutside: EventEmitter<void> = new EventEmitter<void>();
  readonly trackSteps = trackSteps;

  private hoveredColumn: FlameTimeGraphColumn;
  private overlayRef: OverlayRef;
  private tooltipComponent: ComponentRef<FlameTimeGraphTooltipComponent>;
  private expandedGraphComponent: ComponentRef<FlameTimeGraphComponent>;

  constructor(private overlay: Overlay) { super(); }

  onColumnHover(column: FlameTimeGraphColumn): void {
    this.hoveredColumn = column;
    this.tooltipComponent.instance.xSteps = this.xSteps;
    this.tooltipComponent.instance.range = this.ranges[this.checkpoint.columns.indexOf(column)];
    this.tooltipComponent.instance.activeXPointIndex = this.checkpoint.columns.indexOf(column);
    this.tooltipComponent.instance.mean = column.meanTime;
    this.tooltipComponent.instance.max = column.maxTime;
    this.tooltipComponent.instance.calls = column.count;
    this.tooltipComponent.instance.totalTime = column.totalTime;
    this.tooltipComponent.instance.detect();
  }

  attachGraphTooltip(xStepsRef: HTMLDivElement): void {
    if (this.overlayRef?.hasAttached()) {
      this.detachOverlay();
      return;
    }
    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(xStepsRef)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
          offsetX: -11,
          offsetY: 2,
        }]),
    });

    const portal = new ComponentPortal(FlameTimeGraphTooltipComponent);
    this.tooltipComponent = this.overlayRef.attach<FlameTimeGraphTooltipComponent>(portal);
  }

  detachOverlay(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }

  expandCondensedGraph(graphRef: HTMLDivElement, event: MouseEvent): void {
    if (!this.condensedView) {
      return;
    }
    if (this.overlayRef?.hasAttached()) {
      this.detachOverlay();
      return;
    }
    this.overlayRef = this.overlay.create({
      hasBackdrop: true,
      backdropClass: `tracing-overview-overlay-${this.menuCollapsed ? 'big' : 'small'}`,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(graphRef)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
        }]),
    });
    event.stopPropagation();

    const portal = new ComponentPortal(FlameTimeGraphComponent);
    this.expandedGraphComponent = this.overlayRef.attach<FlameTimeGraphComponent>(portal);
    this.expandedGraphComponent.instance.checkpoint = this.checkpoint;
    this.expandedGraphComponent.instance.xSteps = this.xSteps;
    this.expandedGraphComponent.instance.ySteps = this.ySteps;
    this.expandedGraphComponent.instance.visible = false;
    this.expandedGraphComponent.instance.detect();
    this.expandedGraphComponent.instance.onClickOutside
      .pipe(take(1))
      .subscribe(() => this.detachOverlay());

    setTimeout(() => { // for animation
      this.expandedGraphComponent.instance.visible = true;
      this.expandedGraphComponent.instance.detect();
    });
  }

  onOutsideClick(): void {
    this.onClickOutside.emit();
  }
}
