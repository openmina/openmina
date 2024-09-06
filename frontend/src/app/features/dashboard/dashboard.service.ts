import { Injectable } from '@angular/core';
import { catchError, map, Observable, tap } from 'rxjs';
import { DashboardPeer, DashboardPeerStatus } from '@shared/types/dashboard/dashboard.peer';
import { RustService } from '@core/services/rust.service';
import { ONE_MILLION, toReadableDate } from '@openmina/shared';
import { DashboardPeerRpcResponses, DashboardRpcStats } from '@shared/types/dashboard/dashboard-rpc-stats.type';
import { NodesOverviewNode } from '@shared/types/nodes/dashboard/nodes-overview-node.type';
import { NodesOverviewService } from '@nodes/overview/nodes-overview.service';

@Injectable({ providedIn: 'root' })
export class DashboardService {

  constructor(private rust: RustService,
              private nodesOverviewService: NodesOverviewService) { }

  getPeers(): Observable<DashboardPeer[]> {
    return this.rust.get<PeersResponse[]>('/state/peers').pipe(
      map((response: PeersResponse[]) =>
        response.map((peer: PeersResponse) => ({
          peerId: peer.peer_id,
          bestTip: peer.best_tip,
          globalSlot: peer.best_tip_global_slot,
          bestTipDatetime: peer.best_tip_timestamp ? toReadableDate(peer.best_tip_timestamp / ONE_MILLION) : '-',
          bestTipTimestamp: peer.best_tip_timestamp,
          datetime: toReadableDate(peer.time / ONE_MILLION),
          timestamp: peer.time,
          height: peer.best_tip_height,
          status: peer.connection_status,
          address: peer.address,
          requests: 0,
        } as DashboardPeer)),
      ),
      // tap((peers: any) => {
      //   console.log('----------------PEERS----------------');
      //   console.log(peers);
      //   console.log('----------------PEERS----------------');
      // }),
    );
  }

  getTips({ url, name }: { url: string, name: string }): Observable<NodesOverviewNode[]> {
    return this.rust.get<NodesOverviewNode[]>('/stats/sync?limit=1').pipe(
      map((response: NodesOverviewNode[]) => this.nodesOverviewService.mapNodeTipsResponse(response, true, {
        name,
        url,
      })),
      catchError(() => this.nodesOverviewService.mapNodeTipsErrorResponse({
        name,
        url,
      })),
      // tap((peers: any) => {
      //   console.log('----------------SYNC----------------');
      //   console.log(peers);
      //   console.log('----------------SYNC----------------');
      // }),
    );
  }

  getRpcCalls(): Observable<DashboardRpcStats> {
    return this.rust.get<MessageProgressResponse>('/state/message-progress').pipe(
      map((response: MessageProgressResponse) => this.mapMessageProgressResponse(response)),
      // tap((peers: any) => {
      //   console.log('----------------MESSAGES----------------');
      //   console.log(peers);
      //   console.log('----------------MESSAGES----------------');
      // }),
    );
  }

  private mapMessageProgressResponse(response: MessageProgressResponse): DashboardRpcStats {
    const peerResponses = Object.keys(response.messages_stats).map(peerId => ({
      peerId,
      requestsMade: Object
        .keys(response.messages_stats[peerId].responses)
        .reduce((sum: number, curr: string) => sum + response.messages_stats[peerId].responses[curr], 0),
    } as DashboardPeerRpcResponses));

    return {
      peerResponses,
      stakingLedger: response.staking_ledger_sync,
      nextLedger: response.next_epoch_ledger_sync,
      rootLedger: response.root_ledger_sync,
    };
  }
}

interface PeersResponse {
  peer_id: string;
  best_tip: string | null;
  time: number;
  best_tip_height: number;
  best_tip_global_slot: number;
  best_tip_timestamp: number;
  connection_status: DashboardPeerStatus;
  address: string | null;
}

export interface MessageProgressResponse {
  messages_stats: MessagesStats;
  staking_ledger_sync: Estimation;
  next_epoch_ledger_sync: Estimation;
  root_ledger_sync: Estimation;
}

export interface MessagesStats {
  [peerId: string]: {
    current_request: unknown;
    responses: {
      [rpcName: string]: number;
    }
  };
}

export interface Estimation {
  fetched: number;
  estimation: number;
}
