import { ChangeDetectionStrategy, Component, Input, OnChanges, TemplateRef, ViewChild, ViewContainerRef } from '@angular/core';
import { NodesOverviewLedger, NodesOverviewLedgerStepSnarked, NodesOverviewLedgerStepStaged } from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { SecDurationConfig } from '@openmina/shared';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { TemplatePortal } from '@angular/cdk/portal';

@Component({
  selector: 'mina-nodes-overview-ledgers',
  templateUrl: './nodes-overview-ledgers.component.html',
  styleUrls: ['./nodes-overview-ledgers.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'd-flex w-100' },
})
export class NodesOverviewLedgersComponent implements OnChanges {

  @Input() ledgers: NodesOverviewLedger;

  readonly secConfig: SecDurationConfig = {
    onlySeconds: false,
    color: false,
    undefinedAlternative: '-',
  };
  stepsMap: { [key: string]: boolean } = {
    stakingEpoch: false,
    nextEpoch: false,
    root: false,
  };

  @ViewChild('tooltipRef') private tooltipRef: TemplateRef<{ start: number, end: number }>;
  private overlayRef: OverlayRef;

  constructor(private overlay: Overlay,
              private viewContainerRef: ViewContainerRef) { }

  ngOnChanges(): void {
    Object
      .keys(this.ledgers)
      .map(key => key as keyof NodesOverviewLedger)
      .forEach((key: keyof NodesOverviewLedger) => {
        const atLeastOneSnarkerStepHasValue = Object
          .keys(this.ledgers[key].snarked)
          .map(step => step as keyof NodesOverviewLedgerStepSnarked)
          .some(step => this.ledgers[key].snarked[step]);
        const atLeastOneStagedStepHasValue = Object
          .keys(this.ledgers[key].staged)
          .map(step => step as keyof NodesOverviewLedgerStepStaged)
          .some(step => this.ledgers[key].staged[step]);
        this.stepsMap[key] = atLeastOneSnarkerStepHasValue || atLeastOneStagedStepHasValue;
      });
  }

  show(event: MouseEvent, start: number, end: number): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      return;
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(event.target as HTMLElement)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 35,
        }]),
    });
    event.stopPropagation();

    const context = this.tooltipRef
      .createEmbeddedView({ start, end })
      .context;
    const portal = new TemplatePortal(this.tooltipRef, this.viewContainerRef, context);
    this.overlayRef.attach(portal);
  }

  hide(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }
}
