import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { map, Observable } from 'rxjs';
import { NetworkBlock } from '@shared/types/network/blocks/network-block.type';
import { ONE_BILLION, ONE_MILLION, ONE_THOUSAND, toReadableDate } from '@openmina/shared';
import { ConfigService } from '@core/services/config.service';

@Injectable({
  providedIn: 'root',
})
export class NetworkBlocksService {

  constructor(private http: HttpClient,
              private config: ConfigService) { }

  getBlockMessages(height: number): Observable<NetworkBlock[]> {
    return this.http.get<any>(this.config.DEBUGGER + '/block/' + height).pipe(
      map((blocks: any) => this.mapBlocks(blocks)),
    );
  }

  getEarliestBlockHeight(): Observable<number> {
    return this.http.get<any>(this.config.DEBUGGER + '/block/latest').pipe(
      map((blocks: any) => {
        if (!blocks) {
          throw new Error('No blocks found!');
        }
        return blocks.height;
      }),
    );
  }

  private mapBlocks(blocks: any): NetworkBlock[] {
    if (!blocks) {
      return [];
    }

    const allTimestamps = blocks.events.map((block: any) => this.getTimestamp(block.time));
    const fastestTime: bigint = allTimestamps.map((t: string) => BigInt(t)).reduce((t1: bigint, t2: bigint) => t2 < t1 ? t2 : t1);
    return blocks.events.map((block: any, i: number) => ({
      messageKind: block.message_kind,
      producerId: block.producer_id,
      hash: block.hash,
      date: toReadableDate((block.time.secs_since_epoch * ONE_THOUSAND) + block.time.nanos_since_epoch / ONE_MILLION),
      timestamp: allTimestamps[i],
      sender: block.sender_addr,
      receiver: block.receiver_addr,
      height: block.block_height,
      globalSlot: block.global_slot,
      messageId: block.message_id,
      incoming: block.incoming ? 'Incoming' : 'Outgoing',
      [block.incoming ? 'receivedLatency' : 'sentLatency']: Number(BigInt(allTimestamps[i]) - fastestTime) / ONE_BILLION,
    } as NetworkBlock));
  }

  private getTimestamp(time: any): string {
    const secs = time.secs_since_epoch;
    const nano = time.nanos_since_epoch;
    let newNano: string = '' + nano;

    while (newNano.length < 9) {
      newNano = '0' + newNano;
    }

    return secs + '' + newNano;
  }
}
