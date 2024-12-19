import { inject, Injectable } from '@angular/core';
import { NodesOverviewLedger, NodesOverviewLedgerStepState } from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import * as Sentry from '@sentry/angular';
import { NodesOverviewBlock, NodesOverviewNodeBlockStatus } from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import { lastItem, ONE_BILLION } from '@openmina/shared';
import { RustService } from '@core/services/rust.service';
import { getElapsedTime } from '@shared/helpers/date.helper';

@Injectable({
  providedIn: 'root',
})
export class SentryService {

  private readonly rustService: RustService = inject(RustService);
  private ledgerIsSynced: boolean = false;
  private blockIsSynced: boolean = false;
  private ledgerSyncedTime: number;
  private blockSyncedTime: number;

  updateLedgerSyncStatus(ledger: NodesOverviewLedger): void {
    if (this.ledgerIsSynced) {
      return;
    }
    if (ledger.rootStaged.state === NodesOverviewLedgerStepState.SUCCESS) {
      this.ledgerIsSynced = true;
      const syncDetails = {
        stakingLedger: {
          fetchHashes: ledger.stakingEpoch.snarked.fetchHashesDuration + 's',
          fetchAccounts: ledger.stakingEpoch.snarked.fetchAccountsDuration + 's',
        },
        nextEpochLedger: {
          fetchHashes: ledger.nextEpoch.snarked.fetchHashesDuration + 's',
          fetchAccounts: ledger.nextEpoch.snarked.fetchAccountsDuration + 's',
        },
        snarkedRootLedger: {
          fetchHashes: ledger.rootSnarked.snarked.fetchHashesDuration + 's',
          fetchAccounts: ledger.rootSnarked.snarked.fetchAccountsDuration + 's',
        },
        stagedRootLedger: {
          fetchParts: ledger.rootStaged.staged.fetchPartsDuration + 's',
          reconstruct: ledger.rootStaged.staged.reconstructDuration + 's',
        },
      };

      const syncedIn = Math.round((ledger.rootStaged.staged.reconstructEnd - ledger.stakingEpoch.snarked.fetchHashesStart) / ONE_BILLION);
      this.ledgerSyncedTime = syncedIn;

      Sentry.captureMessage(`Ledger synced in ${getElapsedTime(syncedIn)}s`, {
        level: 'info',
        tags: { type: 'webnode', subType: 'sync.ledger' },
        contexts: { ledger: syncDetails },
      });
    }
  }

  updateBlockSyncStatus(blocks: NodesOverviewBlock[], startTime: number): void {
    if (this.blockIsSynced || !this.rustService.activeNodeIsWebNode) {
      return;
    }

    const blocksSynced = blocks.every(b => b.status === NodesOverviewNodeBlockStatus.APPLIED);
    if (blocksSynced && blocks[0]) {
      this.blockIsSynced = true;
      blocks = blocks.slice(1);
      const bestTipBlock = blocks[0].height;
      const root = lastItem(blocks).height;
      this.blockSyncedTime = Math.round((Date.now() - startTime) / 1000);
      Sentry.captureMessage(`Last 290 blocks synced in ${getElapsedTime(this.blockSyncedTime)}s`, {
        level: 'info',
        tags: { type: 'webnode', subType: 'sync.block' },
        contexts: { blocks: { bestTipBlock, root } },
      });

      const syncTotal = this.ledgerSyncedTime + this.blockSyncedTime;
      setTimeout(() => {
        Sentry.captureMessage(`Web Node Synced in ${getElapsedTime(syncTotal)}`, {
          level: 'info',
          tags: { type: 'webnode', subType: 'sync.total' },
        });
      }, 2000);
    }
  }
}
