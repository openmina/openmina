import { Injectable } from '@angular/core';
import { map, Observable } from 'rxjs';
import { RustService } from '@core/services/rust.service';
import {
  MempoolTransaction,
  MempoolTransactionKind,
  SignedCommand,
  ZkappCommand,
} from '@shared/types/mempool/mempool-transaction.type';
import { removeUnicodeEscapes } from '@shared/helpers/transaction.helper';
import { ONE_BILLION } from '@openmina/shared';

@Injectable({
  providedIn: 'root',
})
export class MempoolService {

  constructor(private rust: RustService) { }

  getTransactionPool(limit?: number, from?: number): Observable<{ txs: MempoolTransaction[] }> {
    return this.rust.get<MempoolTransactionResponse[]>('/transaction-pool').pipe(
      // return of({
      // txs: [
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256530,
      //     kind: 'ZKApp command',
      //     txHash: 'bIUYGOUgouYgO68gwra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 1,
      //     nonce: 1,
      //     memo: 'memo1',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256745,
      //     kind: 'Payment',
      //     txHash: '3rg45gqq3gr3wqrg3wqgZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 2,
      //     nonce: 2,
      //     memo: 'memo2',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256847,
      //     kind: 'Delegation',
      //     txHash: 'q3wr4gw34gtq3tg35rMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 3,
      //     nonce: 3,
      //     memo: 'memo3',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256677,
      //     kind: 'ZKApp command',
      //     txHash: '5JthxoBLYoSawergfwegrewVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 4,
      //     nonce: 4,
      //     memo: 'memo4',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256099,
      //     kind: 'Payment',
      //     txHash: '5JthxorweagsgvfergvawergvfnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 5,
      //     nonce: 5,
      //     memo: 'memo5',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Delegation',
      //     txHash: '5r3eg35g5rgwe4g5wg3qr3awrg5RZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 6,
      //     nonce: 6,
      //     memo: 'memo6',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'ZKApp command',
      //     txHash: 'q354ht3hu563qthuyeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 7,
      //     nonce: 7,
      //     memo: 'memo7',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Payment',
      //     txHash: 'q346uh34q3hteu6ExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 8,
      //     nonce: 8,
      //     memo: 'memo8',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Delegation',
      //     txHash: 'hj4q3et6yj34ju6hjtUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 9,
      //     nonce: 9,
      //     memo: 'memo9',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'ZKApp command',
      //     txHash: '3q54uhUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 10,
      //     nonce: 10,
      //     memo: 'memo10',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Payment',
      //     txHash: 'q3uhj6t54e5uUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 11,
      //     nonce: 11,
      //     memo: 'memo11',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Delegation',
      //     txHash: 'q3euhj6BLYoSExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 12,
      //     nonce: 12,
      //     memo: 'memo12',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'ZKApp command',
      //     txHash: 'w465uYoSExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 13,
      //     nonce: 13,
      //     memo: 'memo13',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Payment',
      //     txHash: 'w456ujtoSExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 14,
      //     nonce: 14,
      //     memo: 'memo14',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Delegation',
      //     txHash: '46tewtu4wtExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 15,
      //     nonce: 15,
      //     memo: 'memo15',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'ZKApp command',
      //     txHash: 'je56yhtes456uUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 16,
      //     nonce: 16,
      //     memo: 'memo16',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Payment',
      //     txHash: '46qhet6u4yh4thewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 17,
      //     nonce: 17,
      //     memo: 'memo17',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Delegation',
      //     txHash: '426thy42yExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 18,
      //     nonce: 18,
      //     memo: 'memo18',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'ZKApp command',
      //     txHash: '5435tghSExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 19,
      //     nonce: 19,
      //     memo: 'memo19',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Payment',
      //     txHash: '23q4th654334yxUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 20,
      //     nonce: 20,
      //     memo: 'memo20',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Delegation',
      //     txHash: 'wrht3YoSExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 21,
      //     nonce: 21,
      //     memo: 'memo21',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'ZKApp command',
      //     txHash: '5245436t43rExUMDeDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62hgre082fuehevC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 22,
      //     nonce: 22,
      //     memo: 'memo22',
      //   },
      //   {
      //     status: 'Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Payment',
      //     txHash: '5JthxoBLYoSExUMDe62wra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 23,
      //     nonce: 23,
      //     memo: 'memo23',
      //   },
      //   {
      //     status: 'Not Applicable',
      //     date: '2021-07-14T11:00:00Z',
      //     timestamp: 1626256800,
      //     kind: 'Delegation',
      //     txHash: '5JthxoBL745eDewra8LRZpnDjvCVFfTcRq2EjKL3EwUVTho',
      //     sender: 'B62qif1RJqNXWmv5evC6tGpebdALcHKdCmn4CYFFTGiizauoEftYFWV',
      //     fee: 24,
      //     nonce: 24,
      //     memo: 'memo24',
      //   },
      // ],
      // } as any).pipe(
      //   delay(1000),
      map(data => ({ txs: this.mapTxPoolResponse(data) })),
      map(({ txs }: { txs: any }) => ({ txs: [...txs] })),
    );
  }

  private mapTxPoolResponse(response: MempoolTransactionResponse[]): MempoolTransaction[] {
    return response.map((tx: MempoolTransactionResponse) => {
      if (tx.data.SignedCommand) {
        return {
          kind: MempoolTransactionKind.PAYMENT,
          txHash: tx.hash.join(''),
          sender: tx.data.SignedCommand.payload.common.fee_payer_pk,
          fee: Number(tx.data.SignedCommand.payload.common.fee) / ONE_BILLION,
          amount: Number(tx.data.SignedCommand.payload.body.Payment.amount) / ONE_BILLION,
          nonce: Number(tx.data.SignedCommand.payload.common.nonce),
          memo: removeUnicodeEscapes(tx.data.SignedCommand.payload.common.memo),
          transactionData: tx.data.SignedCommand,
          sentFromStressingTool: tx.data.SignedCommand.payload.common.memo.includes('S.T.'),
          sentByMyBrowser: tx.data.SignedCommand.payload.common.memo.includes(localStorage.getItem('browserId')),
        } as MempoolTransaction;
      } else {
        return {
          kind: MempoolTransactionKind.ZK_APP,
          txHash: tx.hash.join(''),
          sender: tx.data.ZkappCommand.fee_payer.body.public_key,
          fee: Number(tx.data.ZkappCommand.fee_payer.body.fee) / ONE_BILLION,
          amount: null,
          nonce: Number(tx.data.ZkappCommand.fee_payer.body.nonce),
          memo: tx.data.ZkappCommand.memo,
          transactionData: tx.data.ZkappCommand,
          sentFromStressingTool: false,
          sentByMyBrowser: false,
        } as MempoolTransaction;
      }
    });
  }
}


export interface MempoolTransactionResponse {
  data: Data;
  hash: number[];
}

interface Data {
  SignedCommand?: SignedCommand;
  ZkappCommand?: ZkappCommand;
}
