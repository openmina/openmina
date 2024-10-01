import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  OnInit,
  TemplateRef,
  ViewChild,
  ViewContainerRef,
} from '@angular/core';
import { isMobile, ManualDetection } from '@openmina/shared';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import {
  selectBenchmarksActiveWallet,
  selectBenchmarksBlockSending,
  selectBenchmarksRandomWallet,
  selectBenchmarksSendingAmount,
  selectBenchmarksSendingBatch,
  selectBenchmarksSendingFee,
  selectBenchmarksSentTransactionsStats,
  selectBenchmarksWallets,
} from '@benchmarks/wallets/benchmarks-wallets.state';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import {
  BENCHMARKS_WALLETS_CHANGE_AMOUNT,
  BENCHMARKS_WALLETS_CHANGE_FEE,
  BENCHMARKS_WALLETS_CHANGE_TRANSACTION_BATCH,
  BENCHMARKS_WALLETS_SELECT_WALLET,
  BENCHMARKS_WALLETS_SEND_TXS,
  BENCHMARKS_WALLETS_TOGGLE_RANDOM_WALLET,
  BenchmarksWalletsChangeAmount,
  BenchmarksWalletsChangeFee,
  BenchmarksWalletsChangeTransactionBatch,
  BenchmarksWalletsSelectWallet,
  BenchmarksWalletsSendTxs,
  BenchmarksWalletsToggleRandomWallet,
} from '@benchmarks/wallets/benchmarks-wallets.actions';
import { FormBuilder, FormControl, FormGroup } from '@angular/forms';
import { distinctUntilChanged, filter } from 'rxjs';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { TemplatePortal } from '@angular/cdk/portal';
import { BenchmarksWallet } from '@shared/types/benchmarks/wallets/benchmarks-wallet.type';

interface TransactionForm {
  batch: FormControl<number>;
  amount: FormControl<number>;
  fee: FormControl<number>;
}

@UntilDestroy()
@Component({
  selector: 'mina-benchmarks-wallets-toolbar',
  templateUrl: './benchmarks-wallets-toolbar.component.html',
  styleUrls: ['./benchmarks-wallets-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-xl border-bottom' },
})
export class BenchmarksWalletsToolbarComponent extends ManualDetection implements OnInit {

  formGroup: FormGroup<TransactionForm>;
  streamSending: boolean;
  successSentTransactions: number;
  failSentTransactions: number;
  randomWallet: boolean;
  activeWallet: BenchmarksWallet;
  wallets: BenchmarksWallet[];

  private currentBatch: number;
  private walletsLength: number;

  private overlayRef: OverlayRef;

  @ViewChild('walletDropdown') private walletDropdown: TemplateRef<any>;
  @ViewChild('dropdownTrigger') private dropdownTrigger: ElementRef<HTMLDivElement>;

  constructor(private store: Store<MinaState>,
              private formBuilder: FormBuilder,
              private overlay: Overlay,
              private viewContainerRef: ViewContainerRef) { super(); }

  ngOnInit(): void {
    this.initForm();
    this.listenToWalletsChanges();
    this.listenToTransactionChanges();
    this.listenToStressingSendStreaming();
    this.listenToBatchChange();
    this.listenToFeeChange();
    this.listenToAmountChange();
  }

  private listenToWalletsChanges(): void {
    this.store.select(selectBenchmarksWallets)
      .pipe(untilDestroyed(this))
      .subscribe(wallets => {
        this.wallets = wallets;
        this.walletsLength = wallets.length;
        this.detect();
      });
    this.store.select(selectBenchmarksActiveWallet)
      .pipe(untilDestroyed(this))
      .subscribe(activeWallet => {
        this.activeWallet = activeWallet;
        this.detect();
      });
  }

  private listenToTransactionChanges(): void {
    this.store.select(selectBenchmarksSentTransactionsStats)
      .pipe(untilDestroyed(this))
      .subscribe(stats => {
        this.successSentTransactions = stats.success;
        this.failSentTransactions = stats.fail;
        this.detect();
      });
  }

  private listenToBatchChange(): void {
    this.store.select(selectBenchmarksSendingBatch)
      .pipe(untilDestroyed(this))
      .subscribe(batch => {
        this.currentBatch = batch;
        this.formGroup.get('batch').setValue(batch, { emitEvent: false });
        this.detect();
      });
  }

  private listenToFeeChange(): void {
    this.store.select(selectBenchmarksSendingFee)
      .pipe(untilDestroyed(this))
      .subscribe(fee => {
        this.formGroup.get('fee').setValue(Math.abs(fee));
        this.detect();
      });
  }

  private listenToAmountChange(): void {
    this.store.select(selectBenchmarksSendingAmount)
      .pipe(untilDestroyed(this))
      .subscribe(amount => {
        this.formGroup.get('amount').setValue(Math.abs(amount));
        this.detect();
      });
  }

  private listenToStressingSendStreaming(): void {
    this.store.select(selectBenchmarksBlockSending)
      .pipe(untilDestroyed(this))
      .subscribe(streamSending => {
        this.streamSending = streamSending;
        this.detect();
      });
    this.store.select(selectBenchmarksRandomWallet)
      .pipe(untilDestroyed(this))
      .subscribe(randomWallet => {
        this.randomWallet = randomWallet;
        this.detect();
      });
  }

  private initForm(): void {
    this.formGroup = this.formBuilder.group<TransactionForm>({
      batch: new FormControl(0),
      amount: new FormControl(0),
      fee: new FormControl(0),
    });

    this.formGroup.get('batch')
      .valueChanges
      .pipe(
        distinctUntilChanged(),
        filter(v => v !== null),
        untilDestroyed(this),
      )
      .subscribe((value: number) => {
        let payload = Math.ceil(value || 1);
        if (payload > this.walletsLength) {
          payload = this.randomWallet ? this.walletsLength : (this.walletsLength - 1);
        }
        this.formGroup.get('batch').patchValue(payload);
        if (this.currentBatch !== payload) {
          this.store.dispatch<BenchmarksWalletsChangeTransactionBatch>({
            type: BENCHMARKS_WALLETS_CHANGE_TRANSACTION_BATCH,
            payload,
          });
        }
      });
    this.formGroup.get('amount')
      .valueChanges
      .pipe(
        distinctUntilChanged(),
        filter(v => v !== null),
        untilDestroyed(this),
      )
      .subscribe((value: number) => {
        this.store.dispatch<BenchmarksWalletsChangeAmount>({ type: BENCHMARKS_WALLETS_CHANGE_AMOUNT, payload: value });
      });
    this.formGroup.get('fee')
      .valueChanges
      .pipe(
        distinctUntilChanged(),
        filter(v => v !== null),
        untilDestroyed(this),
      )
      .subscribe((value: number) => {
        this.store.dispatch<BenchmarksWalletsChangeFee>({ type: BENCHMARKS_WALLETS_CHANGE_FEE, payload: value });
      });
  }

  send(): void {
    if (!this.streamSending) {
      this.store.dispatch<BenchmarksWalletsSendTxs>({ type: BENCHMARKS_WALLETS_SEND_TXS });
    }
  }

  toggleRandomWallet(): void {
    this.store.dispatch<BenchmarksWalletsToggleRandomWallet>({ type: BENCHMARKS_WALLETS_TOGGLE_RANDOM_WALLET });
  }

  changeWallet(wallet: BenchmarksWallet) {
    this.store.dispatch<BenchmarksWalletsSelectWallet>({ type: BENCHMARKS_WALLETS_SELECT_WALLET, payload: wallet });
    this.detachOverlay();
  }

  openDropdown(event: MouseEvent): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
      return;
    }

    this.overlayRef = this.overlay.create({
      hasBackdrop: false,
      width: isMobile() ? '100%' : undefined,
      height: isMobile() ? '100%' : '350px',
      positionStrategy: this.overlay.position()
        .flexibleConnectedTo(this.dropdownTrigger.nativeElement)
        .withPositions([{
          originX: 'start',
          originY: 'top',
          overlayX: 'start',
          overlayY: 'top',
          offsetY: 35,
        }]),
    });
    event.stopPropagation();

    const portal = new TemplatePortal(this.walletDropdown, this.viewContainerRef);
    this.overlayRef.attach(portal);
  }

  detachOverlay(): void {
    if (this.overlayRef?.hasAttached()) {
      this.overlayRef.detach();
    }
  }
}
