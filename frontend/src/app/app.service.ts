import { Injectable } from '@angular/core';
import { catchError, map, Observable, of } from 'rxjs';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { CONFIG } from '@shared/constants/config';
import { RustService } from '@core/services/rust.service';
import { AppNodeDetails, AppNodeStatus } from '@shared/types/app/app-node-details.type';

@Injectable({
  providedIn: 'root',
})
export class AppService {

  constructor(private rust: RustService) { }

  getActiveNode(nodes: MinaNode[]): Observable<MinaNode> {
    const nodeName = new URL(location.href).searchParams.get('node');
    const configs = nodes;
    const nodeFromURL = configs.find(c => c.name === nodeName) || configs[0];
    return of(nodeFromURL);
  }

  getNodes(): Observable<MinaNode[]> {
    return of([
      ...CONFIG.configs,
      ...(localStorage.getItem('custom_nodes') ? JSON.parse(localStorage.getItem('custom_nodes')) : []),
    ]);
  }

  getActiveNodeDetails(): Observable<AppNodeDetails> {
    return this.rust.get<NodeDetailsResponse>('/status')
      .pipe(
        map((data: NodeDetailsResponse) => ({
          status: this.getStatus(data),
          blockHeight: data.transition_frontier.best_tip.height,
          blockTime: data.transition_frontier.sync.time,
          peers: data.peers.filter(p => p.connection_status === 'Connected').length,
          download: 0,
          upload: 0,
          snarks: data.snark_pool.snarks,
          transactions: data.transaction_pool.transactions,
          chainId: data.chain_id,
        } as AppNodeDetails)),
      );
  }

  private getStatus(data: NodeDetailsResponse): AppNodeStatus {
    if (data.transition_frontier.sync.phase === 'Bootstrap') {
      return AppNodeStatus.BOOTSTRAP;
    }
    if (data.transition_frontier.sync.phase === 'Catchup') {
      return AppNodeStatus.CATCHUP;
    }
    if (data.transition_frontier.sync.phase === 'Synced') {
      return AppNodeStatus.SYNCED;
    }
    return AppNodeStatus.PENDING;
  }
}

export interface NodeDetailsResponse {
  transition_frontier: TransitionFrontier;
  transaction_pool: { transactions: number };
  peers: Peer[];
  snark_pool: SnarkPool;
  chain_id: string | undefined;
}

interface TransitionFrontier {
  best_tip: BestTip;
  sync: Sync;
}

interface BestTip {
  hash: string;
  height: number;
  global_slot: number;
}

interface Sync {
  time: number;
  status: string;
  target: any;
  phase: 'Bootstrap' | 'Catchup' | 'Synced';
}

interface Peer {
  peer_id: string;
  best_tip: string;
  best_tip_height: number;
  best_tip_global_slot: number;
  best_tip_timestamp: number;
  connection_status: string;
  address: string;
  time: number;
}

interface SnarkPool {
  total_jobs: number;
  snarks: number;
}
