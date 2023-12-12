import {
  ChangeDetectionStrategy,
  Component,
  ComponentRef,
  ElementRef,
  Input,
  NgZone,
  OnInit,
  ViewChild
} from '@angular/core';
import { selectActiveNode, selectAppMenu, selectNodes } from '@app/app.state';
import { filter, take } from 'rxjs';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { isMobile, ManualDetection } from '@openmina/shared';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { ComponentPortal } from '@angular/cdk/portal';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { NodePickerComponent } from '@app/layout/node-picker/node-picker.component';

@Component({
  selector: 'mina-server-status',
  templateUrl: './server-status.component.html',
  styleUrls: ['./server-status.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-row align-center' },
})
export class ServerStatusComponent extends ManualDetection implements OnInit {

  @Input() switchForbidden: boolean;

  isMobile: boolean;
  activeNode: MinaNode;

  @ViewChild('overlayOpener') private overlayOpener: ElementRef<HTMLDivElement>;

  private nodes: MinaNode[] = [];
  private overlayRef: OverlayRef;
  private nodePickerComponent: ComponentRef<NodePickerComponent>;

  constructor(private zone: NgZone,
              private overlay: Overlay,
              private store: Store<MinaState>) { super(); }

  ngOnInit(): void {
    this.listenToMenuChange();
    this.listenToNodeChanges();
  }

  private listenToMenuChange(): void {
    this.store.select(selectAppMenu)
      .pipe(filter(menu => menu.isMobile !== this.isMobile))
      .subscribe((menu: AppMenu) => {
        this.isMobile = menu.isMobile;
        this.detect();
      });
  }

  private listenToNodeChanges(): void {
    this.store.select(selectNodes)
      .pipe(filter(nodes => nodes.length > 0))
      .subscribe((nodes: MinaNode[]) => {
        this.nodes = nodes;
        if (this.overlayRef?.hasAttached()) {
          this.detachOverlay();
          this.openNodePicker();
        }
        this.detect();
      });
    this.store.select(selectActiveNode)
      .pipe(filter(Boolean))
      .subscribe((activeNode: MinaNode) => {
        this.activeNode = activeNode;
        this.detect();
      });
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
    this.nodePickerComponent.instance.closeEmitter.pipe(take(1)).subscribe(() => this.detachOverlay());
  }

  detachOverlay(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }
}
