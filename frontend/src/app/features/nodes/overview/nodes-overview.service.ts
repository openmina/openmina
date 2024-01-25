import { Injectable } from '@angular/core';
import { catchError, forkJoin, map, Observable, of } from 'rxjs';
import { HttpClient } from '@angular/common/http';
import { NodesOverviewNode, NodesOverviewNodeKindType } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { hasValue, ONE_BILLION, ONE_MILLION, toReadableDate } from '@openmina/shared';
import { NodesOverviewBlock, NodesOverviewNodeBlockStatus } from '@shared/types/nodes/dashboard/nodes-overview-block.type';
import { NodesOverviewLedger, NodesOverviewLedgerStep, NodesOverviewLedgerStepState } from '@shared/types/nodes/dashboard/nodes-overview-ledger.type';
import { CONFIG } from '@shared/constants/config';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

@Injectable({
  providedIn: 'root',
})
export class NodesOverviewService {

  constructor(private http: HttpClient) { }

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

  getNodeTips(nodeParam: { url: string, name: string }, qp: string = ''): Observable<NodesOverviewNode[]> {
    // return of(JSON.parse(JSON.stringify(mock2()))).pipe(delay(250))
    return this.http.get<any[]>(nodeParam.url + '/stats/sync' + qp)
      .pipe(
        map((response: any[]) => {
          return response.map((node: any) => {
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
              ledgers: this.getLedgers(node.ledgers),
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
            ledgers: this.getLedgers({}),
            blocks: [],
          }]);
        }),
      );
  }

  private getLedgers(ledgers: any): NodesOverviewLedger {
    const ledger = {} as NodesOverviewLedger;
    const getLedgerStep = (step: any): NodesOverviewLedgerStep => {
      const result = {
        state: this.hasAllStepsCompleted(step) ? NodesOverviewLedgerStepState.SUCCESS : this.noneOfStepsCompleted(step) ? NodesOverviewLedgerStepState.PENDING : NodesOverviewLedgerStepState.LOADING,
        snarked: {
          fetchHashesStart: step.snarked.fetch_hashes_start,
          fetchHashesEnd: step.snarked.fetch_hashes_end,
          fetchAccountsStart: step.snarked.fetch_accounts_start,
          fetchAccountsEnd: step.snarked.fetch_accounts_end,
          fetchHashesDuration: this.getDuration(step.snarked.fetch_hashes_start, step.snarked.fetch_hashes_end),
          fetchAccountsDuration: this.getDuration(step.snarked.fetch_accounts_start, step.snarked.fetch_accounts_end),
        },
        staged: {
          fetchPartsStart: step.staged.fetch_parts_start,
          fetchPartsEnd: step.staged.fetch_parts_end,
          reconstructStart: step.staged.reconstruct_start,
          reconstructEnd: step.staged.reconstruct_end,
          fetchPartsDuration: this.getDuration(step.staged.fetch_parts_start, step.staged.fetch_parts_end),
          reconstructDuration: this.getDuration(step.staged.reconstruct_start, step.staged.reconstruct_end),
        },
      } as NodesOverviewLedgerStep;
      result.totalTime = result.snarked.fetchHashesDuration + result.snarked.fetchAccountsDuration + result.staged.fetchPartsDuration + result.staged.reconstructDuration;
      return result;
    };
    if (ledgers.staking_epoch) {
      ledger.stakingEpoch = getLedgerStep(ledgers.staking_epoch);
      if (ledger.stakingEpoch.state !== NodesOverviewLedgerStepState.SUCCESS && (ledgers.staking_epoch.synced || ledgers.next_epoch.synced || ledgers.root.synced)) {
        ledger.stakingEpoch.state = NodesOverviewLedgerStepState.SUCCESS;
      }
    }
    if (ledgers.next_epoch) {
      ledger.nextEpoch = getLedgerStep(ledgers.next_epoch);
      if (ledger.nextEpoch.state !== NodesOverviewLedgerStepState.SUCCESS && (ledgers.next_epoch.synced || ledgers.root.synced)) {
        ledger.nextEpoch.state = NodesOverviewLedgerStepState.SUCCESS;
      }
    }
    if (ledgers.root) {
      ledger.root = getLedgerStep(ledgers.root);
      if (ledger.root.state !== NodesOverviewLedgerStepState.SUCCESS && ledgers.root.synced) {
        ledger.root.state = NodesOverviewLedgerStepState.SUCCESS;
      }
    }
    return ledger;
  }

  private getDuration(start: number, end: number): number | null {
    return (end && start) ? (end - start) / ONE_BILLION : null;
  }

  private hasAllStepsCompleted(step: any): boolean {
    return !!(step.snarked.fetch_hashes_end && step.snarked.fetch_accounts_end && step.staged.fetch_parts_end && step.staged.reconstruct_end);
  }

  private noneOfStepsCompleted(step: any): boolean {
    return !step.snarked.fetch_hashes_start && !step.snarked.fetch_accounts_start && !step.staged.fetch_parts_start && !step.staged.reconstruct_start;
  }
}
