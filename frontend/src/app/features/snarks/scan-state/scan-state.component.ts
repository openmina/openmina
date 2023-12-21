import { ChangeDetectionStrategy, Component, ElementRef, OnDestroy, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  ScanStateClose,
  ScanStateGetBlock, ScanStateInit,
  ScanStateSetActiveJobId,
  ScanStateSidebarResized
} from '@snarks/scan-state/scan-state.actions';
import { getMergedRoute, MergedRoute } from '@openmina/shared';
import { selectActiveNode } from '@app/app.state';
import {
  selectScanStateActiveJobId,
  selectScanStateOpenSidePanel,
  selectScanStateStream
} from '@snarks/scan-state/scan-state.state';
import { distinctUntilChanged, Subscription, take, timer } from 'rxjs';
import { untilDestroyed } from '@ngneat/until-destroy';

@Component({
  selector: 'mina-scan-state',
  templateUrl: './scan-state.component.html',
  styleUrls: ['./scan-state.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ScanStateComponent extends StoreDispatcher implements OnInit, OnDestroy {

  openSidePanel: boolean;

  private heightOrHash: string;
  private activeJobId: string;
  private stream: boolean;
  private timer: Subscription;
  private nodeChanged: boolean = false;

  constructor(public el: ElementRef) { super(); }

  ngOnInit(): void {
    this.dispatch(ScanStateInit);
    this.listenToRoute();
    this.listenToStreamChange();
    this.getBlock();
    this.listenToActiveJobId();
    this.listenToSidePanelChange();
  }

  sideBarResized(): void {
    this.dispatch(ScanStateSidebarResized);
  }

  private getBlock(): void {
    this.select(selectActiveNode, () => {
      this.dispatch(ScanStateGetBlock, this.nodeChanged ? {} : { heightOrHash: this.heightOrHash });
      this.nodeChanged = true;
    });
  }

  private createTimer(): void {
    this.timer = timer(0, 5000)
      .pipe(untilDestroyed(this))
      .subscribe(() => {
        this.dispatch(ScanStateGetBlock, {});
      });
  }

  private listenToStreamChange(): void {
    this.select(selectScanStateStream, (stream: boolean) => {
      this.stream = stream;
      if (stream) {
        this.createTimer();
      } else {
        this.timer?.unsubscribe();
      }
    }, distinctUntilChanged());
  }

  private listenToRoute(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      this.heightOrHash = route.params['heightOrHash'];
      if (route.queryParams['jobId'] && this.activeJobId !== route.queryParams['jobId']) {
        this.dispatch(ScanStateSetActiveJobId, route.queryParams['jobId']);
      }
    });
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['heightOrHash']) {
        this.dispatch(ScanStateSetActiveJobId, route.queryParams['jobId']);
      }
    }, take(1));
  }

  private listenToActiveJobId(): void {
    this.select(selectScanStateActiveJobId, (activeJobId: string) => {
      this.activeJobId = activeJobId;
    });
  }

  private listenToSidePanelChange(): void {
    this.select(selectScanStateOpenSidePanel, open => {
      if (open && !this.openSidePanel) {
        this.openSidePanel = true;
        this.detect();
      } else if (!open && this.openSidePanel) {
        this.openSidePanel = false;
        this.detect();
      }
    });
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.dispatch(ScanStateClose);
  }
}
