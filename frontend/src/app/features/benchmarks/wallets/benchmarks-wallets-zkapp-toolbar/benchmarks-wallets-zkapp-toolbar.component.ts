import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  OnInit,
  TemplateRef,
  ViewChild,
  ViewContainerRef,
} from '@angular/core';
import { UntilDestroy, untilDestroyed } from '@ngneat/until-destroy';
import { FormBuilder, FormControl, FormGroup } from '@angular/forms';
import { isMobile, ManualDetection } from '@openmina/shared';
import { BenchmarksWallet } from '@shared/types/benchmarks/wallets/benchmarks-wallet.type';
import { Overlay, OverlayRef } from '@angular/cdk/overlay';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import {
  selectBenchmarksActiveWallet,
  selectBenchmarksBlockSending,
  selectBenchmarksRandomWallet,
  selectBenchmarksSendingFeeZkapps,
  selectBenchmarksWallets,
  selectBenchmarksZkappsSendingBatch,
} from '@benchmarks/wallets/benchmarks-wallets.state';
import { distinctUntilChanged, filter } from 'rxjs';
import {
  BENCHMARKS_WALLETS_CHANGE_FEE_ZKAPPS,
  BENCHMARKS_WALLETS_CHANGE_ZKAPPS_BATCH,
  BENCHMARKS_WALLETS_SELECT_WALLET,
  BENCHMARKS_WALLETS_SEND_ZKAPPS,
  BENCHMARKS_WALLETS_TOGGLE_RANDOM_WALLET,
  BenchmarksWalletsChangeFeeZkApps,
  BenchmarksWalletsChangeZkAppsBatch,
  BenchmarksWalletsSelectWallet,
  BenchmarksWalletsSendZkApps,
  BenchmarksWalletsToggleRandomWallet,
} from '@benchmarks/wallets/benchmarks-wallets.actions';
import { TemplatePortal } from '@angular/cdk/portal';
import { BenchmarksWalletsZkService } from '@benchmarks/wallets/benchmarks-wallets-zk.service';

interface TransactionForm {
  batch: FormControl<number>;
  fee: FormControl<number>;
}

@UntilDestroy()
@Component({
  selector: 'mina-benchmarks-wallets-zkapp-toolbar',
  templateUrl: './benchmarks-wallets-zkapp-toolbar.component.html',
  styleUrls: ['./benchmarks-wallets-zkapp-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-xl border-bottom' },
})
export class BenchmarksWalletsZkappToolbarComponent extends ManualDetection implements OnInit {

  protected readonly updates$ = this.zkService.updates$;

  formGroup: FormGroup<TransactionForm>;
  streamSending: boolean;
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
              private viewContainerRef: ViewContainerRef,
              private zkService: BenchmarksWalletsZkService) { super(); }

  ngOnInit(): void {
    this.initForm();
    this.listenToWalletsChanges();
    this.listenToStressingSendStreaming();
    this.listenToBatchChange();
    this.listenToFeeChange();
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

  private listenToBatchChange(): void {
    this.store.select(selectBenchmarksZkappsSendingBatch)
      .pipe(untilDestroyed(this))
      .subscribe(batch => {
        this.currentBatch = batch;
        this.formGroup.get('batch').setValue(batch, { emitEvent: false });
        this.detect();
      });
  }

  private listenToFeeChange(): void {
    this.store.select(selectBenchmarksSendingFeeZkapps)
      .pipe(untilDestroyed(this))
      .subscribe(fee => {
        this.formGroup.get('fee').setValue(Math.abs(fee));
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
          this.store.dispatch<BenchmarksWalletsChangeZkAppsBatch>({
            type: BENCHMARKS_WALLETS_CHANGE_ZKAPPS_BATCH,
            payload,
          });
        }
      });
    this.formGroup.get('fee')
      .valueChanges
      .pipe(
        distinctUntilChanged(),
        filter(v => v !== null),
        untilDestroyed(this),
      )
      .subscribe((value: number) => {
        this.store.dispatch<BenchmarksWalletsChangeFeeZkApps>({
          type: BENCHMARKS_WALLETS_CHANGE_FEE_ZKAPPS,
          payload: value,
        });
      });
  }

  send(): void {
    if (!this.streamSending) {
      this.store.dispatch<BenchmarksWalletsSendZkApps>({ type: BENCHMARKS_WALLETS_SEND_ZKAPPS });
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
