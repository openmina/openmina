import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  OnInit,
  TemplateRef,
  ViewChild,
  ViewContainerRef
} from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectDashboardNodes, selectDashboardNodesAndRpcStats } from '@dashboard/dashboard.state';
import {
  NodesOverviewLedger,
  NodesOverviewLedgerStepState
} from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { filter } from 'rxjs';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { ONE_MILLION, SecDurationConfig } from '@openmina/shared';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { TemplatePortal } from '@angular/cdk/portal';
import { DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';

type LedgerConfigMap = {
  stakingEpoch: SecDurationConfig,
  nextEpoch: SecDurationConfig,
  root: SecDurationConfig,
};

@Component({
  selector: 'mina-dashboard-ledger',
  templateUrl: './dashboard-ledger.component.html',
  styleUrls: ['./dashboard-ledger.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardLedgerComponent extends StoreDispatcher implements OnInit, OnDestroy {

  ledgers: NodesOverviewLedger = {
    stakingEpoch: {
      state: NodesOverviewLedgerStepState.PENDING,
      snarked: {
        fetchHashesStart: 0,
        fetchHashesEnd: 0,
        fetchAccountsStart: 0,
        fetchAccountsEnd: 0,
        fetchHashesDuration: null,
        fetchAccountsDuration: null,
      },
      totalTime: null,
    },
    nextEpoch: {
      state: NodesOverviewLedgerStepState.PENDING,
      snarked: {
        fetchHashesStart: 0,
        fetchHashesEnd: 0,
        fetchAccountsStart: 0,
        fetchAccountsEnd: 0,
        fetchHashesDuration: null,
        fetchAccountsDuration: null,
      }, totalTime: null,
    },
    root: {
      state: NodesOverviewLedgerStepState.PENDING,
      snarked: {
        fetchHashesStart: 0,
        fetchHashesEnd: 0,
        fetchAccountsStart: 0,
        fetchAccountsEnd: 0,
        fetchHashesDuration: null,
        fetchAccountsDuration: null,
      },
      staged: {
        fetchPartsStart: 0,
        fetchPartsEnd: 0,
        reconstructStart: 0,
        reconstructEnd: 0,
        fetchPartsDuration: null,
        reconstructDuration: null,
      },
      synced: null,
      totalTime: null,
    }
  };
  progress: string;
  configMap: LedgerConfigMap = {
    stakingEpoch: this.emptyConfig,
    nextEpoch: this.emptyConfig,
    root: this.emptyConfig,
  };

  @ViewChild('tooltipRef') private tooltipRef: TemplateRef<{ start: number, end: number }>;
  private overlayRef: OverlayRef;

  constructor(private overlay: Overlay,
              private viewContainerRef: ViewContainerRef) {
    super();
  }

  ngOnInit(): void {
    this.listenToNodesChanges();
  }

  private listenToNodesChanges(): void {
    this.select(selectDashboardNodesAndRpcStats, ([nodes, rpcStats]: [NodesOverviewNode[], DashboardRpcStats]) => {
      this.ledgers = nodes[0].ledgers;

      const getConfig = (state: NodesOverviewLedgerStepState): SecDurationConfig =>
        state === NodesOverviewLedgerStepState.LOADING ? this.undefinedConfig : this.emptyConfig;

      this.configMap = {
        stakingEpoch: getConfig(this.ledgers.stakingEpoch.state),
        nextEpoch: getConfig(this.ledgers.nextEpoch.state),
        root: getConfig(this.ledgers.root.state),
      };
      this.setProgressTime();
      console.log(rpcStats);
      this.detect();
    }, filter(n => n[0].length > 0));
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

  private get emptyConfig(): SecDurationConfig {
    return {
      includeMinutes: true,
      onlySeconds: false,
      color: false,
    };
  }

  private get undefinedConfig(): SecDurationConfig {
    return {
      includeMinutes: true,
      onlySeconds: false,
      color: false,
      undefinedAlternative: '-',
    };
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.hide();
  }

  private setProgressTime(): void {
    if (!this.ledgers.stakingEpoch.snarked.fetchHashesStart) {
      return;
    }
    if (this.ledgers.root.state === NodesOverviewLedgerStepState.SUCCESS) {
      this.progress = this.calculateProgressTime(this.ledgers.root.synced, 'finished');
    } else {
      this.progress = this.calculateProgressTime(this.ledgers.stakingEpoch.snarked.fetchHashesStart, 'started');
    }
  }

  private calculateProgressTime(timestamp: number, action: string): string {
    timestamp = timestamp / ONE_MILLION;
    // const timestampDate = new Date(timestamp);
    const timezoneOffset = 0;//timestampDate.getTimezoneOffset();

    const millisecondsAgo = Date.now() - timestamp - timezoneOffset * 60 * 1000;
    const minutesAgo = Math.floor(millisecondsAgo / 60000);
    const hoursAgo = Math.floor(minutesAgo / 60);
    const daysAgo = Math.floor(hoursAgo / 24);

    if (daysAgo > 0) {
      return `${action} ${daysAgo}d ago`;
    } else if (hoursAgo > 0) {
      return `${action} ${hoursAgo}h ago`;
    } else if (minutesAgo > 0) {
      return `${action} ${minutesAgo}m ago`;
    } else {
      return `${action} <1m ago`;
    }
  }
}
