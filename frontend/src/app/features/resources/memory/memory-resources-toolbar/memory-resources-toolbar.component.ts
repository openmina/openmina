import { ChangeDetectionStrategy, Component, ElementRef, OnInit, TemplateRef, ViewChild, ViewContainerRef } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import { selectMemoryResourcesState } from '@resources/resources.state';
import { MemoryResourcesState } from '@resources/memory/memory-resources.state';
import { filter } from 'rxjs';
import { MemoryResourcesSetActiveResource, MemoryResourcesSetGranularity, MemoryResourcesSetTreemapView } from '@resources/memory/memory-resources.actions';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { TemplatePortal } from '@angular/cdk/portal';
import { TreemapView } from '@shared/types/resources/memory/treemap-view.type';

@Component({
  selector: 'app-memory-resources-toolbar',
  templateUrl: './memory-resources-toolbar.component.html',
  styleUrls: ['./memory-resources-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MemoryResourcesToolbarComponent extends StoreDispatcher implements OnInit {

  readonly granularities: number[] = [1024, 512, 256, 128, 64, 32, 16, 8, 4];
  readonly treemapOptions: TreemapView[] = Object.values(TreemapView);

  activeResource: MemoryResource;
  breadcrumbs: MemoryResource[];
  granularity: number;
  treemapView: TreemapView;

  @ViewChild('granularityDropdown') private granularityDropdown: TemplateRef<void>;
  @ViewChild('treemapDropdown') private treemapDropdown: TemplateRef<void>;
  @ViewChild('granBtn') private granularityBtnRef: ElementRef<HTMLButtonElement>;
  @ViewChild('treemapBtn') private treemapBtnRef: ElementRef<HTMLButtonElement>;

  private overlayRef: OverlayRef;

  constructor(private viewContainerRef: ViewContainerRef,
              private overlay: Overlay) { super(); }

  ngOnInit(): void {
    this.listenToActiveResource();
  }

  private listenToActiveResource(): void {
    this.select(selectMemoryResourcesState, (state: MemoryResourcesState) => {
      this.activeResource = state.activeResource;
      this.breadcrumbs = state.breadcrumbs;
      this.granularity = state.granularity;
      this.treemapView = state.treemapView;
      this.detect();
    }, filter(s => !!s.resource));
  }


  back(): void {
    if (this.breadcrumbs.length < 2) {
      return;
    }
    this.dispatch(MemoryResourcesSetActiveResource, this.breadcrumbs[this.breadcrumbs.length - 2]);
  }

  openGranularityDropdown(event: MouseEvent): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      width: 86,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(this.granularityBtnRef.nativeElement)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 30,
        }]),
    });
    event.stopPropagation();

    const portal = new TemplatePortal(this.granularityDropdown, this.viewContainerRef);
    this.overlayRef.attach(portal);
  }

  openTreemapDropdown(event: MouseEvent): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      width: 70,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(this.treemapBtnRef.nativeElement)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 30,
        }]),
    });
    event.stopPropagation();

    const portal = new TemplatePortal(this.treemapDropdown, this.viewContainerRef);
    this.overlayRef.attach(portal);
  }

  detach(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }

  setGranularity(granularity: number): void {
    if (granularity === this.granularity) {
      return;
    }
    this.dispatch(MemoryResourcesSetGranularity, granularity);
    this.detach();
  }

  setTreemapView(view: TreemapView): void {
    if (view === this.treemapView) {
      return;
    }
    this.dispatch(MemoryResourcesSetTreemapView, view);
    this.detach();
  }
}
