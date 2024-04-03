import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  OnInit,
  TemplateRef,
  ViewChild,
  ViewContainerRef,
} from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectDashboardNodesAndRpcStats } from '@dashboard/dashboard.state';
import {
  NodesOverviewLedger,
  NodesOverviewLedgerEpochStep,
  NodesOverviewLedgerStepState,
  NodesOverviewRootStagedLedgerStep,
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
  rootSnarked: SecDurationConfig,
  rootStaged: SecDurationConfig,
};

const initialSnarked: NodesOverviewLedgerEpochStep = {
  state: NodesOverviewLedgerStepState.PENDING,
  snarked: {
    fetchHashesStart: 0,
    fetchHashesEnd: 0,
    fetchAccountsStart: 0,
    fetchAccountsEnd: 0,
    fetchHashesDuration: null,
    fetchAccountsDuration: null,
    fetchHashesPassedTime: null,
    fetchAccountsPassedTime: null,
  },
  totalTime: null,
};

const initialStaged: NodesOverviewRootStagedLedgerStep = {
  state: NodesOverviewLedgerStepState.PENDING,
  staged: {
    fetchPartsStart: 0,
    fetchPartsEnd: 0,
    reconstructStart: 0,
    reconstructEnd: 0,
    fetchPartsDuration: null,
    reconstructDuration: null,
    fetchPassedTime: null,
    reconstructPassedTime: null,
  },
  totalTime: null,
};

@Component({
  selector: 'mina-dashboard-ledger',
  templateUrl: './dashboard-ledger.component.html',
  styleUrls: ['./dashboard-ledger.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardLedgerComponent extends StoreDispatcher implements OnInit, OnDestroy {

  protected readonly NodesOverviewLedgerStepState = NodesOverviewLedgerStepState;

  ledgers: NodesOverviewLedger = {
    stakingEpoch: initialSnarked,
    nextEpoch: initialSnarked,
    rootSnarked: initialSnarked,
    rootStaged: initialStaged,
  };
  progress: string;
  configMap: LedgerConfigMap = {
    stakingEpoch: this.emptyConfig,
    nextEpoch: this.emptyConfig,
    rootSnarked: this.emptyConfig,
    rootStaged: this.emptyConfig,
  };
  stakingProgress: number = 0;
  nextProgress: number = 0;
  rootSnarkedProgress: number = 0;
  rootStagedProgress: number = 0;
  totalProgress: number;

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
        rootSnarked: getConfig(this.ledgers.rootSnarked.state),
        rootStaged: getConfig(this.ledgers.rootStaged.state),
      };
      this.setProgressTime();
      this.stakingProgress = rpcStats.stakingLedger?.fetched / rpcStats.stakingLedger?.estimation * 100 || 0;
      this.nextProgress = rpcStats.nextLedger?.fetched / rpcStats.nextLedger?.estimation * 100 || 0;
      this.rootSnarkedProgress = rpcStats.rootLedger?.fetched / rpcStats.rootLedger?.estimation * 100 || 0;
      this.rootStagedProgress = this.ledgers.rootStaged.staged.fetchPartsEnd ? 50 : 0;

      if (this.ledgers.stakingEpoch.state === NodesOverviewLedgerStepState.SUCCESS) {
        this.stakingProgress = 100;
      }
      if (this.ledgers.nextEpoch.state === NodesOverviewLedgerStepState.SUCCESS) {
        this.nextProgress = 100;
      }
      if (this.ledgers.rootSnarked.state === NodesOverviewLedgerStepState.SUCCESS) {
        this.rootSnarkedProgress = 100;
      }
      if (this.ledgers.rootStaged.state === NodesOverviewLedgerStepState.SUCCESS) {
        this.rootStagedProgress = 100;
      }
      this.totalProgress = (this.stakingProgress + this.nextProgress + this.rootSnarkedProgress + this.rootStagedProgress) / 4;
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
    if (this.ledgers.rootStaged.state === NodesOverviewLedgerStepState.SUCCESS) {
      this.progress = this.calculateProgressTime(this.ledgers.rootStaged.staged.reconstructEnd, 'finished');
    } else {
      this.progress = this.calculateProgressTime(this.ledgers.stakingEpoch.snarked.fetchHashesStart, 'started');
    }
  }

  private calculateProgressTime(timestamp: number, action: string): string {
    timestamp = timestamp / ONE_MILLION;
    const millisecondsAgo = Date.now() - timestamp * 60 * 1000;
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
