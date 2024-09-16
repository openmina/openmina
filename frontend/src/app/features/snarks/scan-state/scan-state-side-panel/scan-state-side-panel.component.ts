import { ChangeDetectionStrategy, Component, OnInit, TemplateRef, ViewChild, ViewContainerRef } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { ScanStateSetActiveLeaf, ScanStateToggleSidePanel } from '@snarks/scan-state/scan-state.actions';
import { selectScanStateActiveLeaf, selectScanStateBlock } from '@snarks/scan-state/scan-state.state';
import { ScanStateBlock } from '@shared/types/snarks/scan-state/scan-state-block.type';
import { filter } from 'rxjs';
import { Router } from '@angular/router';
import { Routes } from '@shared/enums/routes.enum';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { TemplatePortal } from '@angular/cdk/portal';
import { ScanStateLeaf } from '@shared/types/snarks/scan-state/scan-state-leaf.type';
import { AppSelectors } from '@app/app.state';
import { getFeaturesConfig } from '@shared/constants/config';

@Component({
  selector: 'mina-scan-state-side-panel',
  templateUrl: './scan-state-side-panel.component.html',
  styleUrls: ['./scan-state-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ScanStateSidePanelComponent extends StoreDispatcher implements OnInit {

  activeStep: number = 0;
  block: ScanStateBlock = {} as ScanStateBlock;
  workingSnarkersLength: number;
  hasError: boolean = false;
  activeLeaf: ScanStateLeaf;
  workingPoolEnabled: boolean = false;

  @ViewChild('navDropdown') private dropdown: TemplateRef<void>;

  private overlayRef: OverlayRef;

  constructor(private router: Router,
              private overlay: Overlay,
              private viewContainerRef: ViewContainerRef) { super(); }

  ngOnInit(): void {
    this.listenToBlockChange();
    this.listenToActiveJobID();
    this.listenToActiveNode();
  }

  toggleSidePanel(): void {
    this.dispatch(ScanStateToggleSidePanel);
    this.removeActiveJobId();
  }

  private listenToBlockChange(): void {
    this.select(selectScanStateBlock, (block: ScanStateBlock) => {
      this.block = block;
      this.workingSnarkersLength = block.workingSnarkers.filter(s => s.leafs.length).length;
      this.hasError = block.workingSnarkers.length !== this.workingSnarkersLength;
      this.detect();
    }, filter(Boolean));
  }

  private listenToActiveJobID(): void {
    this.select(selectScanStateActiveLeaf, (leaf: ScanStateLeaf) => {
      this.activeLeaf = leaf;
      if (this.activeLeaf) {
        this.activeStep = 1;
      }
      this.detect();
    });
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, (node) => {
      this.workingPoolEnabled = getFeaturesConfig(node).snarks?.includes('work-pool');
      this.detect();
    });
  }

  openSnarkerDetails(leaf: ScanStateLeaf): void {
    this.dispatch(ScanStateSetActiveLeaf, { ...leaf, scrolling: true });
    this.routeToJobId(leaf.bundle_job_id);
  }

  removeActiveJobId(): void {
    this.dispatch(ScanStateSetActiveLeaf, undefined);
    this.routeToJobId(undefined);
    this.activeStep = 0;
  }

  private routeToJobId(jobId: string | undefined): void {
    this.router.navigate([Routes.SNARKS, Routes.SCAN_STATE, this.block.height], {
      queryParamsHandling: 'merge',
      queryParams: { jobId },
    });
  }

  openNavDropdown(event: MouseEvent): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      return;
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      width: 200,
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(event.target as HTMLElement)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 35,
          offsetX: -12,
        }]),
    });
    event.stopPropagation();

    const portal = new TemplatePortal(this.dropdown, this.viewContainerRef);
    this.overlayRef.attach(portal);
  }

  detach(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }

  goToWorkPool(): void {
    const queryParams = this.router.parseUrl(this.router.url).queryParams;
    let url = `${window.location.origin}/${Routes.SNARKS}/${Routes.WORK_POOL}/${this.activeLeaf.bundle_job_id}`;
    url += `?node=${queryParams['node']}`;
    window.open(url, '_blank');
  }
}
