import {
  ChangeDetectionStrategy,
  Component,
  ComponentRef,
  ElementRef,
  Input,
  OnInit,
  TemplateRef,
  ViewChild,
  ViewContainerRef,
} from '@angular/core';
import { AppSelectors } from '@app/app.state';
import { filter, take } from 'rxjs';
import { isDesktop, isMobile, MAX_WIDTH_700, ONE_MILLION } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { ComponentPortal, TemplatePortal } from '@angular/cdk/portal';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { NodePickerComponent } from '@app/layout/node-picker/node-picker.component';
import { NewNodeComponent } from '@app/layout/new-node/new-node.component';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { AppNodeDetails, AppNodeStatus } from '@shared/types/app/app-node-details.type';
import { getTimeDiff } from '@shared/helpers/date.helper';
import { CONFIG } from '@shared/constants/config';
import { animate, state, style, transition, trigger } from '@angular/animations';
import { BreakpointObserver, BreakpointState } from '@angular/cdk/layout';

@Component({
  selector: 'mina-server-status',
  templateUrl: './server-status.component.html',
  styleUrls: ['./server-status.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row align-center' },
  animations: [
    trigger('fadeIn', [
      state('void', style({
        opacity: 0,
        transform: 'translateY(-10px)',
      })),
      transition(':enter', [
        animate('200ms ease-out', style({
          opacity: 1,
          transform: 'translateY(0)',
        })),
      ]),
    ]),
  ],
})
export class ServerStatusComponent extends StoreDispatcher implements OnInit {

  protected readonly AppNodeStatus = AppNodeStatus;
  protected readonly canAddNodes = CONFIG.globalConfig?.canAddNodes;

  @Input() switchForbidden: boolean;

  isMobile: boolean;
  activeNode: MinaNode;
  details: AppNodeDetails;
  isOnline: boolean;
  blockTimeAgo: string;
  hideNodeStats: boolean = CONFIG.hideNodeStats;

  @ViewChild('overlayOpener') private overlayOpener: ElementRef<HTMLDivElement>;

  private nodes: MinaNode[] = [];
  private tooltipOverlayRef: OverlayRef;
  private nodePickerOverlayRef: OverlayRef;
  private nodePickerComponent: ComponentRef<NodePickerComponent>;
  private newNodeComponent: ComponentRef<NewNodeComponent>;

  constructor(private overlay: Overlay,
              private viewContainerRef: ViewContainerRef) { super(); }

  ngOnInit(): void {
    this.listenToMenuChange();
    this.listenToNodeChanges();
  }

  private listenToMenuChange(): void {
    this.select(AppSelectors.menu, (menu: AppMenu) => {
      this.isMobile = menu.isMobile;
      this.detect();
    }, filter(menu => menu.isMobile !== this.isMobile));
  }

  private listenToNodeChanges(): void {
    this.select(AppSelectors.nodes, (nodes: MinaNode[]) => {
      this.nodes = nodes;
      if (this.tooltipOverlayRef?.hasAttached()) {
        this.detachNodeOverlay();
      }
      this.detect();
    }, filter(nodes => nodes.length > 0));
    this.select(AppSelectors.activeNode, (activeNode: MinaNode) => {
      this.activeNode = activeNode;
      this.detect();
    }, filter(Boolean));
    this.select(AppSelectors.activeNodeDetails, (activeNodeDetails: AppNodeDetails) => {
      this.details = activeNodeDetails;
      this.isOnline = ![AppNodeStatus.PENDING, AppNodeStatus.OFFLINE].includes(activeNodeDetails?.status);
      this.blockTimeAgo = getTimeDiff(activeNodeDetails?.blockTime / ONE_MILLION).diff;
      this.detect();
    });
  }

  openTooltipDropdown(anchor: HTMLDivElement, template: TemplateRef<void>): void {
    if (this.details?.status === AppNodeStatus.PENDING) {
      return;
    }
    this.tooltipOverlayRef = this.overlay.create({
      hasBackdrop: false,
      width: isDesktop() ? 188 : 205,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(anchor)
        .withPositions([{
          originX: 'start',
          originY: 'bottom',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 10,
        }]),
    });

    const portal = new TemplatePortal(template, this.viewContainerRef);
    this.tooltipOverlayRef.attach(portal);
  }

  openNodePicker(event?: MouseEvent): void {
    event?.stopImmediatePropagation();
    if (this.nodePickerOverlayRef?.hasAttached()) {
      this.nodePickerOverlayRef.detach();
      return;
    }

    this.nodePickerOverlayRef = this.overlay.create({
      hasBackdrop: false,
      width: '100%',
      maxWidth: '300px',
      minWidth: isMobile() ? '100%' : '220px',
      scrollStrategy: this.overlay.scrollStrategies.close(),
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(this.overlayOpener.nativeElement)
        .withPositions([{
          originX: 'end',
          originY: 'bottom',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 8,
          offsetX: isMobile() ? 0 : -10,
        }]),
    });

    const portal = new ComponentPortal(NodePickerComponent);
    this.nodePickerComponent = this.nodePickerOverlayRef.attach<NodePickerComponent>(portal);
    this.nodePickerComponent.instance.nodes = this.nodes;
    this.nodePickerComponent.instance.filteredNodes = this.nodes;
    this.nodePickerComponent.instance.activeNode = this.activeNode;
    this.nodePickerComponent.instance.closeEmitter.pipe(take(1)).subscribe((addNewNode: boolean) => {
      this.detachNodeOverlay();
      if (addNewNode) {
        this.openAddNewNodeOverlay();
      }
    });
  }

  detachTooltipOverlay(): void {
    if (this.tooltipOverlayRef?.hasAttached()) {
      this.tooltipOverlayRef.detach();
    }
  }

  detachNodeOverlay(): void {
    if (this.nodePickerOverlayRef?.hasAttached()) {
      this.nodePickerOverlayRef.detach();
    }
  }

  private openAddNewNodeOverlay(): void {
    if (this.nodePickerOverlayRef?.hasAttached()) {
      this.nodePickerOverlayRef.detach();
      return;
    }

    this.nodePickerOverlayRef = this.overlay.create({
      hasBackdrop: true,
      width: '100%',
      maxWidth: '900px',
      height: '100%',
      maxHeight: '650px',
      scrollStrategy: this.overlay.scrollStrategies.block(),
      positionStrategy: this.overlay.position()
        .global()
        .centerHorizontally()
        .centerVertically(),
    });

    const portal = new ComponentPortal(NewNodeComponent);
    this.newNodeComponent = this.nodePickerOverlayRef.attach<NewNodeComponent>(portal);
    this.newNodeComponent.instance.closeEmitter.pipe(take(1)).subscribe(() => this.detachNodeOverlay());
  }
}
