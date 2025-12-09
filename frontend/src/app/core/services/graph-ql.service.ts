import { Injectable } from '@angular/core';
import { catchError, concatMap, delay, forkJoin, from, map, Observable, of, throwError, toArray } from 'rxjs';
import { HttpClient } from '@angular/common/http';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { AccountsResponse } from '@benchmarks/wallets/benchmarks-wallets.service';
import { MempoolTransactionResponse, MempoolTransactionResponseKind } from '@mempool/mempool.service';
import { SignedCommand, ZkappCommand } from '@shared/types/mempool/mempool-transaction.type';


@Injectable({
  providedIn: 'root',
})
export class GraphQLService {

  private url: string;

  constructor(private http: HttpClient) { }

  changeGraphQlProvider(node: MinaNode): void {
    this.url = node.url + '/graphql';
  }

  get<T>(path: string, data: any): Observable<T> {
    const gqlBuilder = GQL_QUERY_MAP[path];
    if (gqlBuilder) {
      const queryData = gqlBuilder(data);
      return this.query<any>(queryData).pipe(
        map((response: any): T => OCAML_TO_RUST_MAP[path](response) as T),
      );
    }
    throw new Error(`No GQL mapping for path: ${path}`);
  }

  post<T>(path: string, data: any): Observable<T> {
    const gqlBuilder = GQL_QUERY_MAP[path];
    if (!gqlBuilder) {
      throw new Error(`No GQL mapping for path: ${path}`);
    }

    const isArray = Array.isArray(data);
    const dataArray = isArray ? data : [data];
    const DELAY_MS = 100;

    return from(dataArray).pipe(
      concatMap((item: any, index: number) => {
        const queryData = gqlBuilder(item);
        return this.mutation<any>(queryData, item).pipe(
          map((response: any) => OCAML_TO_RUST_MAP[path](response)),
          catchError((err) => of(OCAML_TO_RUST_MAP[path](err))),
          delay(index > 0 ? DELAY_MS : 0),
        );
      }),
      toArray(),
      map((results: T[]) => {
        return isArray ? results : results[0];
      }),
    ) as Observable<T>;
  }

  private query<T>(queryData: GqlQuery, variables?: { [key: string]: any }): Observable<T> {
    const query = `query ${queryData.queryName} ${queryData.query}`;
    return this.performGqlRequest(query, variables);
  }

  private mutation<T>(queryData: GqlQuery, variables?: { [key: string]: any }): Observable<T> {
    const query = `mutation ${queryData.queryName} ${queryData.query}`;
    return this.performGqlRequest(query, variables);
  }

  private performGqlRequest<T>(query: string, variables: { [key: string]: any }): Observable<T> {
    return this.http
      .post<{ data: T }>(
        this.url,
        { query, variables },
        { headers: { 'Content-Type': 'application/json' } },
      )
      .pipe(
        catchError((err: Error) => {
          return throwError(() => err);
        }),
        map((response: { data: T }) => {
          if ((response as any).errors) {
            return response as any;
          }
          if (response.data) {
            return response.data;
          }
        }),
      );
  }
}

export type GqlQuery = { queryName: string, query: string };
export type GqlMap = { [key: string]: (data: any) => GqlQuery };

const GQL_QUERY_MAP: GqlMap = {
  '/status': () => getStatus(),
  '/accounts': (data: any) => getAccounts(data),
  '/transaction-pool': () => getPooledUserCommands(),
  '/best-chain-user-commands': () => getBestChainUserCommands(),
  '/send-payment': () => getSendPayment(),
};

function getStatus(): GqlQuery {
  const query = `{
  daemonStatus {
    syncStatus
    uptime
    version
    getPeers {
      host
      libp2pPort
      peerId
    }
  }}`;
  return { queryName: 'GetStatus', query };
}

function getAccounts(data: any): GqlQuery {
  let query = '{';
  const accounts = data.accounts;
  accounts.forEach((account: any, i: number) => query += `account${i}: account(publicKey: "${account.publicKey}") { publicKey nonce balance { liquid } }, `);
  query += '}';
  return { queryName: 'GetAccounts', query };
}

function getPooledUserCommands(): GqlQuery {
  const query = `{ pooledUserCommands { ... on UserCommandPayment {
    id
    hash
    kind
    nonce
    validUntil
    token
    amount
    feeToken
    fee
    memo
    isDelegation
    from
    to
    failureReason
  } } }`;
  return { queryName: 'GetPooledUserCommands', query };
}

function getBestChainUserCommands(): GqlQuery {
  const query = `{
    bestChain {
      transactions {
        userCommands{
         id
         hash
         kind
         nonce
         source {
          publicKey
         }
         receiver {
          publicKey
         }
         fee
         amount
         memo
         from
        }
        zkappCommands
      }
    }
  }`;
  return { queryName: 'GetBestChainUserCommands', query };
}

function getSendPayment(): GqlQuery {
  const query = `
    ($fee:UInt64!, $amount:UInt64!,
    $to: PublicKey!, $from: PublicKey!, $nonce:UInt32, $memo: String,
    $valid_until: UInt32, $signature_scalar: String!, $signature_field: String!
    ) {
      sendPayment(
        input: {
          fee: $fee,
          amount: $amount,
          to: $to,
          from: $from,
          memo: $memo,
          nonce: $nonce,
          validUntil: $valid_until
        },
        signature: {
          field: $signature_field, scalar: $signature_scalar
        }) {
        payment {
          amount
          fee
          feeToken
          from
          hash
          id
          isDelegation
          memo
          nonce
          kind
          to
        }
      }
    }`;
  return { queryName: 'SendPayment', query };
}

const OCAML_TO_RUST_MAP: { [key: string]: any } = {
  '/status': statusMapper,
  '/accounts': accountsResponseMapper,
  '/transaction-pool': pooledUserCommandsMapper,
  '/best-chain-user-commands': bestChainUserCommandsMapper,
  '/send-payment': sendPaymentMapper,
};

function accountsResponseMapper(data: any): AccountsResponse[] {
  return Object.keys(data)
    .filter(key => key.startsWith('account') && /account\d+/.test(key))
    .map(key => data[key])
    .filter(account => account !== null && account !== undefined)
    .map(account => ({
      public_key: account.publicKey,
      balance: account.balance.liquid,
      nonce: account.nonce || '0',
    }));
}

function pooledUserCommandsMapper(data: any): MempoolTransactionResponse[] {
  const result: MempoolTransactionResponse[] = [];
  // Process pooled user commands (payments and delegations)
  data.pooledUserCommands.forEach((cmd: any) => {
    const signedCommand: SignedCommand = {
      payload: {
        common: {
          fee: cmd.fee,
          fee_payer_pk: cmd.from,
          nonce: cmd.nonce.toString(),
          valid_until: cmd.validUntil,
          memo: cmd.memo,
        },
        body: [
          'Payment',
          {
            receiver_pk: cmd.to,
            amount: cmd.isDelegation ? undefined : cmd.amount,
          },
        ],
      },
      signer: cmd.from,
      signature: cmd.signature,
    };

    // Create the mempool transaction response
    const mempoolTx: MempoolTransactionResponse = {
      data: [MempoolTransactionResponseKind.SignedCommand, signedCommand],
      hash: cmd.hash,
    };

    result.push(mempoolTx);
  });

  // Process pooled zkapp commands

  return result;
}

// Complete mapper implementation
function bestChainUserCommandsMapper(data: any): Array<[MempoolTransactionResponseKind, SignedCommand | ZkappCommand]> {
  const result: Array<[MempoolTransactionResponseKind, SignedCommand | ZkappCommand]> = [];
  data.bestChain.forEach((block: any) => {
    if (!block.transactions) {
      return;
    }

    block.transactions.userCommands.forEach((cmd: any) => {
      const signedCommand: SignedCommand = {
        payload: {
          common: {
            fee: cmd.fee,
            fee_payer_pk: cmd.source?.publicKey,
            nonce: cmd.nonce,
            valid_until: cmd.validUntil,
            memo: cmd.memo,
          },
          body: [
            'Payment',
            {
              receiver_pk: cmd.receiver?.publicKey,
              amount: cmd.amount,
            },
          ],
        },
        signer: cmd.source?.publicKey,
        signature: cmd.signature,
      };
      result.push([MempoolTransactionResponseKind.SignedCommand, signedCommand]);
    });
    block.transactions.zkappCommands.forEach((cmd: any) => {
      const zkappCommand: ZkappCommand = null;
      // result.push([MempoolTransactionResponseKind.ZkappCommand, zkappCommand]);
    });
  });
  return result;
}

function sendPaymentMapper(data: any): any {
  if (data.errors?.[0]) {
    return [data.errors?.[0].path[0], data.errors?.[0].message];
  }
  return data;
}

function statusMapper(data: any): any {
  console.log(data);
  return {};
}
