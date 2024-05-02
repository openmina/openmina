import { ChangeDetectionStrategy, Component, ComponentRef, ElementRef, Input, OnInit, ViewChild } from '@angular/core';
import { AppSelectors } from '@app/app.state';
import { filter, take } from 'rxjs';
import { isMobile } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { ComponentPortal } from '@angular/cdk/portal';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { NodePickerComponent } from '@app/layout/node-picker/node-picker.component';
import { NewNodeComponent } from '@app/layout/new-node/new-node.component';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';

@Component({
  selector: 'mina-server-status',
  templateUrl: './server-status.component.html',
  styleUrls: ['./server-status.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row align-center' },
})
export class ServerStatusComponent extends StoreDispatcher implements OnInit {

  @Input() switchForbidden: boolean;

  isMobile: boolean;
  activeNode: MinaNode;

  @ViewChild('overlayOpener') private overlayOpener: ElementRef<HTMLDivElement>;

  private nodes: MinaNode[] = [];
  private overlayRef: OverlayRef;
  private nodePickerComponent: ComponentRef<NodePickerComponent>;
  private newNodeComponent: ComponentRef<NewNodeComponent>;

  constructor(private overlay: Overlay) { super(); }

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
      if (this.overlayRef?.hasAttached()) {
        this.detachOverlay();
        this.openNodePicker();
      }
      this.detect();
    }, filter(nodes => nodes.length > 0));
    this.select(AppSelectors.activeNode, (activeNode: MinaNode) => {
      this.activeNode = activeNode;
      this.detect();
    }, filter(Boolean));
  }

  openNodePicker(event?: MouseEvent): void {
    event?.stopImmediatePropagation();
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      return;
    }

    this.overlayRef = this.overlay.create({
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
    this.nodePickerComponent = this.overlayRef.attach<NodePickerComponent>(portal);
    this.nodePickerComponent.instance.nodes = this.nodes;
    this.nodePickerComponent.instance.filteredNodes = this.nodes;
    this.nodePickerComponent.instance.activeNode = this.activeNode;
    this.nodePickerComponent.instance.closeEmitter.pipe(take(1)).subscribe((addNewNode: boolean) => {
      this.detachOverlay();
      if (addNewNode) {
        this.openAddNewNodeOverlay();
      }
    });
  }

  detachOverlay(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }

  private openAddNewNodeOverlay(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      return;
    }

    this.overlayRef = this.overlay.create({
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
    this.newNodeComponent = this.overlayRef.attach<NewNodeComponent>(portal);
    this.newNodeComponent.instance.closeEmitter.pipe(take(1)).subscribe(() => this.detachOverlay());
  }
}
