import { Injectable } from '@angular/core';
import { map, Observable } from 'rxjs';
import { RustService } from '@core/services/rust.service';
import {
  MempoolTransaction,
  MempoolTransactionKind,
  SignedCommand,
  ZkappCommand,
} from '@shared/types/mempool/mempool-transaction.type';
import { decodeMemo, removeUnicodeEscapes } from '@shared/helpers/transaction.helper';
import { ONE_BILLION } from '@openmina/shared';

@Injectable({
  providedIn: 'root',
})
export class MempoolService {

  constructor(private rust: RustService) { }

  getTransactionPool(limit?: number, from?: number): Observable<{ txs: MempoolTransaction[] }> {
    return this.rust.get<MempoolTransactionResponse[]>('/transaction-pool').pipe(
      map(data => ({ txs: this.mapTxPoolResponse(data) })),
      map(({ txs }: { txs: any }) => ({ txs: [...txs] })),
    );
  }

  private mapTxPoolResponse(response: MempoolTransactionResponse[]): MempoolTransaction[] {
    return response.map((tx: MempoolTransactionResponse) => {
      switch (tx.data[0]) {
        case MempoolTransactionResponseKind.SignedCommand:
          const memo = decodeMemo(tx.data[1].payload.common.memo);
          return {
            kind: MempoolTransactionKind.PAYMENT,
            txHash: tx.hash.join(''),
            sender: tx.data[1].payload.common.fee_payer_pk,
            fee: Number(tx.data[1].payload.common.fee),
            amount: Number(tx.data[1].payload.body[1].amount) / ONE_BILLION,
            nonce: Number(tx.data[1].payload.common.nonce),
            memo: removeUnicodeEscapes(memo),
            transactionData: tx.data[1],
            sentFromStressingTool: memo.includes('S.T.'),
            sentByMyBrowser: memo.includes(localStorage.getItem('browserId')),
          } as MempoolTransaction;
        case MempoolTransactionResponseKind.ZkappCommand:
          const zkapp = tx.data[1] as ZkappCommand;
          const zkMemo = decodeMemo(zkapp.memo);
          return {
            kind: MempoolTransactionKind.ZK_APP,
            txHash: tx.hash.join(''),
            sender: zkapp.fee_payer.body.public_key,
            fee: Number(zkapp.fee_payer.body.fee),
            amount: null,
            nonce: Number(zkapp.fee_payer.body.nonce),
            memo: removeUnicodeEscapes(zkMemo),
            transactionData: tx.data[1],
            sentFromStressingTool: zkMemo.includes('S.T.'),
            sentByMyBrowser: zkMemo.includes(localStorage.getItem('browserId')),
          } as MempoolTransaction;
      }
    });
  }
}


export interface MempoolTransactionResponse {
  data: [MempoolTransactionResponseKind, SignedCommand | ZkappCommand];
  hash: number[];
}

export enum MempoolTransactionResponseKind {
  SignedCommand = 'Signed_command',
  ZkappCommand = 'Zkapp_command',
}
