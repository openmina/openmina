import { Injectable } from '@angular/core';
import { MinaState, selectMinaState } from '@app/app.setup';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { Effect } from '@openmina/shared';
import { forkJoin, map, switchMap } from 'rxjs';
import { Store } from '@ngrx/store';
import {
  BENCHMARKS_WALLETS_GET_ALL_TXS, BENCHMARKS_WALLETS_GET_ALL_TXS_SUCCESS,
  BENCHMARKS_WALLETS_GET_WALLETS,
  BENCHMARKS_WALLETS_GET_WALLETS_SUCCESS,
  BENCHMARKS_WALLETS_SEND_TX_SUCCESS,
  BENCHMARKS_WALLETS_SEND_TX_SYNCED,
  BENCHMARKS_WALLETS_SEND_TXS,
  BenchmarksWalletsActions, BenchmarksWalletsGetWallets,
  BenchmarksWalletsSendTxs,
  BenchmarksWalletsSendTxSynced,
} from '@benchmarks/wallets/benchmarks-wallets.actions';
import { BenchmarksWalletsService } from '@benchmarks/wallets/benchmarks-wallets.service';
import { catchErrorAndRepeat } from '@shared/constants/store-functions';
import { MinaErrorType } from '@shared/types/error-preview/mina-error-type.enum';
import { MinaRustBaseEffect } from '@shared/base-classes/mina-rust-base.effect';
import { MempoolService } from '@app/features/mempool/mempool.service';

@Injectable({
  providedIn: 'root',
})
export class BenchmarksWalletsEffects extends MinaRustBaseEffect<BenchmarksWalletsActions> {

  readonly getWallets$: Effect;
  readonly sendTxs$: Effect;
  readonly sendTxSynced$: Effect;
  readonly sendTxSuccess$: Effect;
  readonly getAllTxs$: Effect;

  constructor(private actions$: Actions,
              private benchmarksService: BenchmarksWalletsService,
              private mempoolService: MempoolService,
              store: Store<MinaState>) {

    super(store, selectMinaState);

    this.getWallets$ = createEffect(() => this.actions$.pipe(
      ofType(BENCHMARKS_WALLETS_GET_WALLETS),
      this.latestActionState<BenchmarksWalletsGetWallets>(),
      switchMap(({ action }) => this.benchmarksService.getAccounts().pipe(
        switchMap(payload => {
          const actions = [];
          if (action.payload?.initialRequest) {
            actions.push({ type: BENCHMARKS_WALLETS_GET_ALL_TXS });
          }
          actions.push({ type: BENCHMARKS_WALLETS_GET_WALLETS_SUCCESS, payload });
          return actions;
        }),
      )),
      catchErrorAndRepeat(MinaErrorType.GENERIC, BENCHMARKS_WALLETS_GET_WALLETS_SUCCESS, []),
    ));

    this.sendTxs$ = createEffect(() => this.actions$.pipe(
      ofType(BENCHMARKS_WALLETS_SEND_TXS),
      this.latestActionState<BenchmarksWalletsSendTxs>(),
      map(() => ({ type: BENCHMARKS_WALLETS_SEND_TX_SYNCED })),
    ));

    this.sendTxSynced$ = createEffect(() => this.actions$.pipe(
      ofType(BENCHMARKS_WALLETS_SEND_TX_SYNCED),
      this.latestActionState<BenchmarksWalletsSendTxSynced>(),
      switchMap(({ state }) => this.benchmarksService.sendTransactions(state.benchmarks.wallets.txsToSend)),
      map(payload => ({ type: BENCHMARKS_WALLETS_SEND_TX_SUCCESS, payload })),
    ));

    this.sendTxSuccess$ = createEffect(() => this.actions$.pipe(
      ofType(BENCHMARKS_WALLETS_SEND_TX_SUCCESS),
      map(() => ({ type: BENCHMARKS_WALLETS_GET_WALLETS })),
    ));

    this.getAllTxs$ = createEffect(() => this.actions$.pipe(
      ofType(BENCHMARKS_WALLETS_GET_ALL_TXS),
      switchMap(() =>
        forkJoin([
          this.mempoolService.getTransactionPool(),
          this.benchmarksService.getAllIncludedTransactions(),
        ]),
      ),
      map(([{ txs }, includedTxs]) => ({
        type: BENCHMARKS_WALLETS_GET_ALL_TXS_SUCCESS,
        payload: { mempoolTxs: txs, includedTxs },
      })),
      catchErrorAndRepeat(MinaErrorType.GENERIC, BENCHMARKS_WALLETS_GET_ALL_TXS_SUCCESS, {
        memPoolTxs: [],
        includedTxs: [],
      }),
    ));
  }
}
