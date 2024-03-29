import { Injectable } from '@angular/core';
import { catchError, forkJoin, map, Observable, of } from 'rxjs';
import { HttpClient } from '@angular/common/http';
import { NodesOverviewNode, NodesOverviewNodeKindType } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { hasValue, ONE_BILLION, ONE_MILLION, toReadableDate } from '@openmina/shared';
import {
  NodesOverviewBlock,
  NodesOverviewNodeBlockStatus
} from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import {
  NodesOverviewLedger,
  NodesOverviewLedgerEpochStep,
  NodesOverviewLedgerStepState,
  NodesOverviewRootLedgerStep,
  NodesOverviewStagedLedgerStep,
} from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { CONFIG } from '@shared/constants/config';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { NodesOverviewResync } from '@shared/types/nodes/dashboard/nodes-overview-resync.type';

@Injectable({
  providedIn: 'root',
})
export class NodesOverviewService {

  constructor(private http: HttpClient) {
  }

  getNodes(): Observable<NodesOverviewNode[]> {
    const nodes = CONFIG.configs;
    return forkJoin(
      nodes.map((node: MinaNode) => {
        return this.getNodeTips({ url: node.url, name: node.name }, '?limit=1');
      }),
    ).pipe(
      map((nodes: NodesOverviewNode[][]) => nodes.map(n => n[0])),
    );
  }

  getNodeTips(nodeParam: {
    url: string,
    name: string
  }, qp: string = '', onlyOne: boolean = false): Observable<NodesOverviewNode[]> {
    return this.http.get<any[]>(nodeParam.url + '/stats/sync' + qp)
      .pipe(
        map((response: any[]) => {
          if (response.length === 0) {
            throw new Error('Empty response');
          }
          return response
            .slice(0, onlyOne ? 1 : response.length)
            .map((node: any) => {
              const blocks = node.blocks.map((block: any) => {
                return {
                  globalSlot: block.global_slot,
                  height: block.height,
                  hash: block.hash,
                  predHash: block.pred_hash,
                  status: block.status,
                  fetchStart: block.fetch_start,
                  fetchEnd: block.fetch_end,
                  applyStart: block.apply_start,
                  applyEnd: block.apply_end,
                  fetchDuration: this.getDuration(block.fetch_start, block.fetch_end),
                  applyDuration: this.getDuration(block.apply_start, block.apply_end),
                } as NodesOverviewBlock;
              });
              if (blocks.length) {
                blocks[0].isBestTip = true;
              }
              return {
                name: nodeParam.name,
                kind: hasValue(node.synced) ? NodesOverviewNodeKindType.SYNCED : node.kind,
                bestTipReceived: toReadableDate(node.best_tip_received / ONE_MILLION),
                bestTipReceivedTimestamp: node.best_tip_received / ONE_MILLION,
                bestTip: node.blocks[0]?.hash,
                height: node.blocks[0]?.height,
                globalSlot: node.blocks[0]?.global_slot,
                appliedBlocks: node.blocks.filter((block: any) => block.status === NodesOverviewNodeBlockStatus.APPLIED).length,
                applyingBlocks: node.blocks.filter((block: any) => block.status === NodesOverviewNodeBlockStatus.APPLYING).length,
                missingBlocks: node.blocks.filter((block: any) => block.status === NodesOverviewNodeBlockStatus.MISSING).length,
                fetchedBlocks: node.blocks.filter((block: any) => block.status === NodesOverviewNodeBlockStatus.FETCHED).length,
                fetchingBlocks: node.blocks.filter((block: any) => block.status === NodesOverviewNodeBlockStatus.FETCHING).length,
                ledgers: this.getLedgers(node.ledgers, node.synced),
                resyncs: this.getResyncs(node.resyncs),
                blocks,
              } as NodesOverviewNode;
            });
        }),
        catchError(err => {
          return of([{
            name: nodeParam.name,
            kind: NodesOverviewNodeKindType.OFFLINE,
            bestTipReceived: '-',
            bestTipReceivedTimestamp: 0,
            bestTip: '-',
            height: undefined,
            globalSlot: 0,
            appliedBlocks: 0,
            applyingBlocks: 0,
            missingBlocks: 0,
            fetchedBlocks: 0,
            fetchingBlocks: 0,
            ledgers: this.getLedgers({}, null),
            resyncs: [],
            blocks: [],
          }]);
        }),
      );
  }

  private getLedgers(ledgers: any, synced: number): NodesOverviewLedger {
    const ledger = {} as NodesOverviewLedger;

    const epochLedger = this.getLedgerStep(ledgers.root);
    if (!ledgers.root) {
      ledger.root = {
        ...epochLedger,
        staged: {
          fetchPartsStart: null,
          fetchPartsEnd: null,
          reconstructStart: null,
          reconstructEnd: null,
          fetchPartsDuration: null,
          reconstructDuration: null,
        } as NodesOverviewStagedLedgerStep,
      } as NodesOverviewRootLedgerStep;
    } else {
      ledger.root = {
        ...epochLedger,
        state: this.noneOfStepsCompleted(ledgers.root) && !ledgers.root.staged.fetch_parts_start && !ledgers.root.staged.reconstruct_start
          ? NodesOverviewLedgerStepState.PENDING
          : NodesOverviewLedgerStepState.LOADING,
        staged: {
          fetchPartsStart: ledgers.root.staged.fetch_parts_start,
          fetchPartsEnd: ledgers.root.staged.fetch_parts_end,
          reconstructStart: ledgers.root.staged.reconstruct_start,
          reconstructEnd: ledgers.root.staged.reconstruct_end,
          fetchPartsDuration: this.getDuration(ledgers.root.staged.fetch_parts_start, ledgers.root.staged.fetch_parts_end),
          reconstructDuration: this.getDuration(ledgers.root.staged.reconstruct_start, ledgers.root.staged.reconstruct_end),
        },
        synced: null,
      };
      if (synced) {
        ledger.root.synced = synced;
        ledger.root.state = NodesOverviewLedgerStepState.SUCCESS;
        ledger.root.totalTime = epochLedger.snarked.fetchHashesDuration + epochLedger.snarked.fetchAccountsDuration + ledger.root.staged.fetchPartsDuration + ledger.root.staged.reconstructDuration;
      }
    }

    ledger.nextEpoch = this.getLedgerStep(ledgers.next_epoch);
    if (ledger.root.state !== NodesOverviewLedgerStepState.PENDING) {
      ledger.nextEpoch.state = NodesOverviewLedgerStepState.SUCCESS;
      ledger.nextEpoch.totalTime = ledger.nextEpoch.snarked.fetchHashesDuration + ledger.nextEpoch.snarked.fetchAccountsDuration;
    }

    ledger.stakingEpoch = this.getLedgerStep(ledgers.staking_epoch);
    if (ledger.nextEpoch.state !== NodesOverviewLedgerStepState.PENDING) {
      ledger.stakingEpoch.state = NodesOverviewLedgerStepState.SUCCESS;
      ledger.stakingEpoch.totalTime = ledger.stakingEpoch.snarked.fetchHashesDuration + ledger.stakingEpoch.snarked.fetchAccountsDuration;
    }

    return ledger;
  }

  private getLedgerStep(step: any): NodesOverviewLedgerEpochStep {
    if (!step) {
      return {
        state: NodesOverviewLedgerStepState.PENDING,
        snarked: {
          fetchHashesStart: null,
          fetchHashesEnd: null,
          fetchAccountsStart: null,
          fetchAccountsEnd: null,
          fetchHashesDuration: null,
          fetchAccountsDuration: null,
        },
      } as NodesOverviewLedgerEpochStep;
    }
    return {
      state: this.noneOfStepsCompleted(step) ? NodesOverviewLedgerStepState.PENDING : NodesOverviewLedgerStepState.LOADING,
      snarked: {
        fetchHashesStart: step.snarked.fetch_hashes_start,
        fetchHashesEnd: step.snarked.fetch_hashes_end,
        fetchAccountsStart: step.snarked.fetch_accounts_start,
        fetchAccountsEnd: step.snarked.fetch_accounts_end,
        fetchHashesDuration: this.getDuration(step.snarked.fetch_hashes_start, step.snarked.fetch_hashes_end),
        fetchAccountsDuration: this.getDuration(step.snarked.fetch_accounts_start, step.snarked.fetch_accounts_end),
      },
    } as NodesOverviewLedgerEpochStep;
  };

  private getDuration(start: number, end: number): number | null {
    if (!(end && start)) {
      return null;
    }
    return Math.ceil((end - start) / ONE_BILLION);
  }

  private hasAllStepsCompleted(step: any): boolean {
    return !!(step.snarked.fetch_hashes_end && step.snarked.fetch_accounts_end);
  }

  private noneOfStepsCompleted(step: any): boolean {
    return !step.snarked.fetch_hashes_start && !step.snarked.fetch_accounts_start;
  }

  private getResyncs(resyncs: any[]): NodesOverviewResync[] {
    return resyncs.map((resync: any) => {
      const kind = typeof resync.kind === 'string' ? resync.kind : Object.keys(resync.kind)[0];
      return {
        kind: kind.replace(/([A-Z])/g, ' $1').trim(),
        description: typeof resync.kind === 'string' ? undefined : Object.values(resync.kind)[0]?.toString(),
        time: resync.time,
      };
    });
  }
}
