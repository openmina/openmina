import { Injectable } from '@angular/core';
import { catchError, forkJoin, map, Observable, of } from 'rxjs';
import { HttpClient } from '@angular/common/http';
import { NodesOverviewNode, NodesOverviewNodeKindType } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { hasValue, ONE_BILLION, ONE_MILLION, ONE_THOUSAND, toReadableDate } from '@openmina/shared';
import {
  NodesOverviewBlock,
  NodesOverviewNodeBlockStatus,
} from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import {
  NodesOverviewLedger,
  NodesOverviewLedgerEpochStep,
  NodesOverviewLedgerStepState,
} from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { NodeDetailsResponse } from '@app/app.service';

@Injectable({
  providedIn: 'root',
})
export class NodesOverviewService {

  constructor(private http: HttpClient) {}

  getNodeTips(nodeParam: { url: string, name: string }, qp: string = ''): Observable<NodesOverviewNode[]> {
    return forkJoin([
      this.http.get<any[]>(nodeParam.url + '/stats/sync' + qp),
      this.http.get<NodeDetailsResponse>(nodeParam.url + '/status' + qp),
    ])
      .pipe(
        map((response: [any, NodeDetailsResponse]) => this.mapNodeTipsResponse(response, nodeParam)),
        catchError(() => this.mapNodeTipsErrorResponse(nodeParam)),
      );
  }

  public mapNodeTipsErrorResponse(nodeParam: { url: string; name: string }): Observable<NodesOverviewNode[]> {
    return of([{
      name: nodeParam.name,
      kind: NodesOverviewNodeKindType.OFFLINE,
      snarks: 0,
      transactions: 0,
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
      ledgers: this.getLedgers({}),
      blocks: [],
    }]);
  }

  public mapNodeTipsResponse([syncTips, nodeDetails]: [any, NodeDetailsResponse | undefined], nodeParam: {
    url: string;
    name: string
  }): NodesOverviewNode[] {
    if (syncTips.length === 0) {
      throw new Error('Empty response');
    }
    return syncTips
      .map((node: any) => {
        const blocks = node.blocks.map((block: any) => ({
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
        } as NodesOverviewBlock));
        if (blocks.length) {
          blocks[0].isBestTip = true;
        }
        return {
          name: nodeParam.name,
          kind: nodeDetails ? nodeDetails.transition_frontier.sync.phase : (hasValue(node.synced) ? NodesOverviewNodeKindType.SYNCED : node.kind),
          snarks: nodeDetails?.snark_pool.snarks,
          transactions: nodeDetails?.transaction_pool.transactions,
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
          ledgers: this.getLedgers(node.ledgers),
          blocks,
        } as NodesOverviewNode;
      });
  }

  private getKind(node: any): NodesOverviewNodeKindType {
    if (node.kind === 'Synced') {
      return NodesOverviewNodeKindType.SYNCED;
    } else {
      if (!node.blocks[0]?.hash) {
        return NodesOverviewNodeKindType.BOOTSTRAP;
      }
      return NodesOverviewNodeKindType.CATCHUP;
    }
  }

  private getLedgers(ledgers: any): NodesOverviewLedger {
    const ledger = {} as NodesOverviewLedger;

    const epochLedger = this.getSnarkedLedgerStep(ledgers.root);
    if (!ledgers.root) {
      ledger.rootSnarked = epochLedger;
      ledger.rootStaged = {
        state: NodesOverviewLedgerStepState.PENDING,
        staged: {
          fetchPartsStart: null,
          fetchPartsEnd: null,
          fetchPartsDuration: null,
          fetchPassedTime: null,
          reconstructStart: null,
          reconstructEnd: null,
          reconstructDuration: null,
          reconstructPassedTime: null,
        },
        totalTime: null,
      };
    } else {
      ledger.rootSnarked = epochLedger;
      ledger.rootStaged = {
        state: !ledgers.root.staged.fetch_parts_start && !ledgers.root.staged.reconstruct_start ? NodesOverviewLedgerStepState.PENDING : NodesOverviewLedgerStepState.LOADING,
        staged: {
          fetchPartsStart: ledgers.root.staged.fetch_parts_start,
          fetchPartsEnd: ledgers.root.staged.fetch_parts_end,
          reconstructStart: ledgers.root.staged.reconstruct_start,
          reconstructEnd: ledgers.root.staged.reconstruct_end,
          fetchPartsDuration: this.getDuration(ledgers.root.staged.fetch_parts_start, ledgers.root.staged.fetch_parts_end),
          reconstructDuration: this.getDuration(ledgers.root.staged.reconstruct_start, ledgers.root.staged.reconstruct_end),
          fetchPassedTime: this.getPassed(ledgers.root.staged.fetch_parts_start),
          reconstructPassedTime: this.getPassed(ledgers.root.staged.reconstruct_start),
        },
        totalTime: null,
      };
      if (ledger.rootStaged.state !== NodesOverviewLedgerStepState.PENDING) {
        ledger.rootSnarked.state = NodesOverviewLedgerStepState.SUCCESS;
        ledger.rootSnarked.totalTime = ledger.rootSnarked.snarked.fetchHashesDuration + ledger.rootSnarked.snarked.fetchAccountsDuration;
      }
      if (ledger.rootStaged.staged.reconstructEnd) {
        ledger.rootStaged.state = NodesOverviewLedgerStepState.SUCCESS;
        ledger.rootStaged.totalTime = ledger.rootStaged.staged.fetchPartsDuration + ledger.rootStaged.staged.reconstructDuration;
      }
    }

    ledger.nextEpoch = this.getSnarkedLedgerStep(ledgers.next_epoch);
    if (ledger.rootSnarked.state !== NodesOverviewLedgerStepState.PENDING) {
      ledger.nextEpoch.state = NodesOverviewLedgerStepState.SUCCESS;
      ledger.nextEpoch.totalTime = ledger.nextEpoch.snarked.fetchHashesDuration + ledger.nextEpoch.snarked.fetchAccountsDuration;
    }

    ledger.stakingEpoch = this.getSnarkedLedgerStep(ledgers.staking_epoch);
    if (ledger.nextEpoch.state !== NodesOverviewLedgerStepState.PENDING) {
      ledger.stakingEpoch.state = NodesOverviewLedgerStepState.SUCCESS;
      ledger.stakingEpoch.totalTime = ledger.stakingEpoch.snarked.fetchHashesDuration + ledger.stakingEpoch.snarked.fetchAccountsDuration;
    }

    return ledger;
  }

  private getSnarkedLedgerStep(step: any): NodesOverviewLedgerEpochStep {
    if (!step) {
      return {
        state: NodesOverviewLedgerStepState.PENDING,
        snarked: {
          fetchHashesStart: null,
          fetchHashesEnd: null,
          fetchHashesDuration: null,
          fetchHashesPassedTime: null,
          fetchAccountsStart: null,
          fetchAccountsEnd: null,
          fetchAccountsDuration: null,
          fetchAccountsPassedTime: null,
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
        fetchHashesPassedTime: this.getPassed(step.snarked.fetch_hashes_start),
        fetchAccountsPassedTime: this.getPassed(step.snarked.fetch_accounts_start),
      },
    } as NodesOverviewLedgerEpochStep;
  };

  private getDuration(start: number, end: number): number | null {
    if (!(end && start)) {
      return null;
    }
    return Math.ceil((end - start) / ONE_BILLION);
  }

  private getPassed(start: number): number | null {
    if (!start) {
      return null;
    }
    return Math.ceil((Date.now() / ONE_THOUSAND) - (start / ONE_BILLION));
  }

  private noneOfStepsCompleted(step: any): boolean {
    return !step.snarked.fetch_hashes_start && !step.snarked.fetch_accounts_start;
  }
}
