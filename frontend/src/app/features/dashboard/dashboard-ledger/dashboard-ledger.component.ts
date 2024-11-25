import { ChangeDetectionStrategy, Component, OnDestroy, OnInit, TemplateRef, ViewChild, ViewContainerRef } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { selectDashboardNodesAndRpcStats } from '@dashboard/dashboard.state';
import {
  NodesOverviewLedger,
  NodesOverviewLedgerEpochStep,
  NodesOverviewLedgerStepState,
  NodesOverviewRootStagedLedgerStep,
} from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { ONE_MILLION, SecDurationConfig } from '@openmina/shared';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { TemplatePortal } from '@angular/cdk/portal';
import { DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';
import { AppSelectors } from '@app/app.state';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

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
  isWebNode: boolean;

  @ViewChild('tooltipRef') private tooltipRef: TemplateRef<{ start: number, end: number }>;
  private overlayRef: OverlayRef;

  constructor(private overlay: Overlay,
              private viewContainerRef: ViewContainerRef) {
    super();
  }

  ngOnInit(): void {
    this.listenToActiveNode();
    this.listenToNodesChanges();
  }

  private listenToActiveNode(): void {
    this.select(AppSelectors.activeNode, (node: MinaNode) => {
      this.isWebNode = node.isWebNode;
    });
  }

  remainingStakingLedger: number;
  private previousStakingLedgerDownloaded: number;
  remainingNextLedger: number;
  private previousNextLedgerDownloaded: number;
  remainingRootSnarkedLedger: number;
  private previousRootSnarkedLedgerDownloaded: number;
  remainingRootStagedLedgerFetchParts: number;
  private previousRootStagedLedgerDownloaded: number;
  remainingReconstruct: number = 20;
  private reconstructTimer: any;

  private listenToNodesChanges(): void {
    this.select(selectDashboardNodesAndRpcStats, ([nodes, rpcStats]: [NodesOverviewNode[], DashboardRpcStats]) => {
      if (nodes.length === 0) {
        this.ledgers = {
          stakingEpoch: initialSnarked,
          nextEpoch: initialSnarked,
          rootSnarked: initialSnarked,
          rootStaged: initialStaged,
        };
        this.configMap = {
          stakingEpoch: this.emptyConfig,
          nextEpoch: this.emptyConfig,
          rootSnarked: this.emptyConfig,
          rootStaged: this.emptyConfig,
        };
        this.progress = undefined;
        this.stakingProgress = 0;
        this.nextProgress = 0;
        this.rootSnarkedProgress = 0;
        this.rootStagedProgress = 0;
        this.totalProgress = 0;
      } else {
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

        const stakingLedgerStartTime = this.ledgers.stakingEpoch.snarked.fetchHashesStart / ONE_MILLION;
        const currentStakingLedgerDownloaded = rpcStats.stakingLedger?.fetched;
        if (
          this.previousStakingLedgerDownloaded && currentStakingLedgerDownloaded &&
          stakingLedgerStartTime < Date.now()
        ) {
          const timeSinceDownloadStarted = Date.now() - stakingLedgerStartTime;
          const remaining = rpcStats.stakingLedger?.estimation - currentStakingLedgerDownloaded;
          const remainingTime = (remaining / currentStakingLedgerDownloaded) * timeSinceDownloadStarted;
          this.remainingStakingLedger = Math.floor(remainingTime / 1000);
        }
        this.previousStakingLedgerDownloaded = currentStakingLedgerDownloaded;

        const nextLedgerStartTime = this.ledgers.nextEpoch.snarked.fetchHashesStart / ONE_MILLION;
        const currentNextLedgerDownloaded = rpcStats.nextLedger?.fetched;
        if (
          this.previousNextLedgerDownloaded && currentNextLedgerDownloaded &&
          nextLedgerStartTime < Date.now()
        ) {
          const timeSinceDownloadStarted = Date.now() - nextLedgerStartTime;
          const remaining = rpcStats.nextLedger?.estimation - currentNextLedgerDownloaded;
          const remainingTime = (remaining / currentNextLedgerDownloaded) * timeSinceDownloadStarted;
          this.remainingNextLedger = Math.floor(remainingTime / 1000);
        }
        this.previousNextLedgerDownloaded = currentNextLedgerDownloaded;

        const rootSnarkedLedgerStartTime = this.ledgers.rootSnarked.snarked.fetchHashesStart / ONE_MILLION;
        const currentRootSnarkedLedgerDownloaded = rpcStats.snarkedRootLedger?.fetched;
        if (
          this.previousRootSnarkedLedgerDownloaded && currentRootSnarkedLedgerDownloaded &&
          rootSnarkedLedgerStartTime < Date.now()
        ) {
          const timeSinceDownloadStarted = Date.now() - rootSnarkedLedgerStartTime;
          const remaining = rpcStats.snarkedRootLedger?.estimation - currentRootSnarkedLedgerDownloaded;
          const remainingTime = (remaining / currentRootSnarkedLedgerDownloaded) * timeSinceDownloadStarted;
          this.remainingRootSnarkedLedger = Math.floor(remainingTime / 1000);
        }
        this.previousRootSnarkedLedgerDownloaded = currentRootSnarkedLedgerDownloaded;

        if (this.isWebNode) {
          const rootStagedLedgerStartTime = this.ledgers.rootStaged.staged.fetchPartsStart / ONE_MILLION;
          const currentRootStagedLedgerDownloaded = rpcStats.stagedRootLedger?.fetched;
          if (
            this.previousRootStagedLedgerDownloaded && currentRootStagedLedgerDownloaded &&
            rootStagedLedgerStartTime < Date.now()
          ) {
            const timeSinceDownloadStarted = Date.now() - rootStagedLedgerStartTime;
            const remaining = rpcStats.stagedRootLedger?.estimation - currentRootStagedLedgerDownloaded;
            const remainingTime = (remaining / currentRootStagedLedgerDownloaded) * timeSinceDownloadStarted;
            this.remainingRootStagedLedgerFetchParts = Math.floor(remainingTime / 1000);
          }
          this.previousRootStagedLedgerDownloaded = currentRootStagedLedgerDownloaded;
        }

        this.stakingProgress = rpcStats.stakingLedger?.fetched / rpcStats.stakingLedger?.estimation * 100 || 0;
        this.nextProgress = rpcStats.nextLedger?.fetched / rpcStats.nextLedger?.estimation * 100 || 0;
        this.rootSnarkedProgress = rpcStats.snarkedRootLedger?.fetched / rpcStats.snarkedRootLedger?.estimation * 100 || 0;

        this.rootStagedProgress = 0;
        if (this.ledgers.rootStaged.staged.fetchPartsEnd) {
          this.rootStagedProgress += 50;
        }
        if (this.ledgers.rootStaged.staged.reconstructEnd) {
          this.rootStagedProgress += 50;
        }
        if (this.rootStagedProgress < 100 && this.isWebNode && this.ledgers.rootStaged.staged.fetchPartsEnd && !this.reconstructTimer) {
          this.startTimerForReconstruct();
        } else if (this.rootStagedProgress === 100) {
          clearTimeout(this.reconstructTimer);
        }

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
      }
      this.detect();
    });
  }

  startTimerForReconstruct(): void {
    this.reconstructTimer = setInterval(() => {
      this.remainingReconstruct = this.remainingReconstruct - 1;
      this.detect();
      if (this.remainingReconstruct === Math.floor(Math.random() * 2) + 2) {
        clearTimeout(this.reconstructTimer);
      }
    }, 1000);
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
    timestamp = Math.ceil(timestamp / ONE_MILLION);
    const millisecondsAgo = Date.now() - timestamp;
    const minutesAgo = Math.floor(millisecondsAgo / 1000 / 60);
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
