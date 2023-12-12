import { Injectable } from '@angular/core';
import { RustService } from '@core/services/rust.service';
import { catchError, forkJoin, map, Observable, of, switchMap, tap } from 'rxjs';
import { ScanStateBlock } from '@shared/types/snarks/scan-state/scan-state-block.type';
import { HttpClient } from '@angular/common/http';
import { ScanStateLeaf, ScanStateLeafStatus } from '@shared/types/snarks/scan-state/scan-state-leaf.type';
import { ScanStateTree } from '@shared/types/snarks/scan-state/scan-state-tree.type';
import { ScanStateWorkingSnarker } from '@shared/types/snarks/scan-state/scan-state-working-snarker.type';
import { CONFIG } from '@shared/constants/config';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

@Injectable({
  providedIn: 'root'
})
export class ScanStateService {

  private snarkers: ScanStateWorkingSnarker[] = [];

  constructor(private rust: RustService,
              private http: HttpClient) { }

  getScanState(heightOrHash?: string | number): Observable<ScanStateBlock> {
    const url = this.rust.URL + '/scan-state/summary/' + (heightOrHash ?? '');

    // return of(scanStateRust).pipe(
    return this.http.get<any>(url).pipe(
      switchMap((response: any[]) => {
        if (this.snarkers.length) {
          return of(response);
        }
        return forkJoin(
          CONFIG.configs.map((node: MinaNode) =>
            this.http.get<{ public_key: string }>(node.url + '/snarker/config')
              .pipe(
                map(r => ({
                  hash: r.public_key,
                  name: node.name,
                  url: node.url,
                  local: node.url === this.rust.URL,
                  leafs: []
                })),
                catchError(err => {
                  let message = err.message;
                  if (!message.includes('Http failure') && err.statusCode < 400) {
                    message = 'Frontend parsing error';
                  }
                  return of({
                    hash: '',
                    name: node.name,
                    local: node.url === this.rust.URL,
                    url: node.url,
                    leafs: [],
                    error: message,
                  });
                }),
              )
          )
        ).pipe(
          tap((snarkers: ScanStateWorkingSnarker[]) => this.snarkers = snarkers),
          map(() => response),
        );
      }),

      map((response: any) => this.mapScanState(response)),
    );
  }

  private mapScanState(response: any): ScanStateBlock {
    const trees: ScanStateTree[] = response.scan_state.reverse().map((tree: any[], treeIndex: number) => {
      return {
        availableJobs: tree.filter(leaf => leaf.status === ScanStateLeafStatus.Todo).length,
        ongoing: tree.filter(leaf => leaf.commitment && !leaf.snark).length,
        notIncludedSnarks: tree.filter(leaf => leaf.snark && leaf.status === ScanStateLeafStatus.Pending).length,
        completedSnarks: tree.filter(leaf => leaf.snark && leaf.status === ScanStateLeafStatus.Done).length,
        leafs: tree.map((t, jobIndex: number) => ({ ...t, treeIndex, jobIndex })),
        empty: tree.filter(l => !l.job?.kind).length,
        coinbase: tree.filter(l => l.job?.kind === 'Coinbase').length,
        feeTransfer: tree.filter(l => l.job?.kind === 'FeeTransfer').length,
        payment: tree.filter(l => l.job?.kind === 'Payment').length,
        zkApp: tree.filter(l => l.job?.kind === 'Zkapp').length,
        merge: tree.filter(l => l.job?.kind === 'Merge').length,
      } as ScanStateTree;
    });
    const workingLeafs = trees.flatMap(tree => tree.leafs.filter(l => l.commitment && !l.snark));

    this.snarkers = this.snarkers.map(s => ({ ...s, local: s.url === this.rust.URL, leafs: [] }));

    workingLeafs.forEach((leaf: ScanStateLeaf) => {
      const snarker = this.snarkers.find(s => s.hash === leaf.commitment.commitment.snarker);
      if (snarker) {
        snarker.leafs.push(leaf);
      }
    });

    return {
      hash: response.block.hash,
      height: response.block.height,
      globalSlot: response.block.global_slot,
      completedWorks: response.block.completed_works,
      transactions: response.block.transactions,
      workingSnarkers: this.snarkers,
      trees
    };
  }
}
