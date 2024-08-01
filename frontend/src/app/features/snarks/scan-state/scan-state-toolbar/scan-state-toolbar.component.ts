import { ChangeDetectionStrategy, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import {
  selectScanStateBlock, selectScanStateHighlightSnarkPool,
  selectScanStateOpenSidePanel,
  selectScanStateStream,
  selectScanStateTreeView,
} from '@snarks/scan-state/scan-state.state';
import { ScanStateBlock } from '@shared/types/snarks/scan-state/scan-state-block.type';
import { debounceTime, delay, distinctUntilChanged, filter, fromEvent, map, mergeMap, of } from 'rxjs';
import {
  ScanStateGetBlock, ScanStateHighlightSnarkPool,
  ScanStatePause,
  ScanStateStart,
  ScanStateToggleSidePanel,
  ScanStateToggleTreeView,
} from '@snarks/scan-state/scan-state.actions';
import { FormBuilder, FormGroup } from '@angular/forms';
import { untilDestroyed } from '@ngneat/until-destroy';
import { NumberInput } from '@angular/cdk/coercion';

@Component({
  selector: 'mina-scan-state-toolbar',
  templateUrl: './scan-state-toolbar.component.html',
  styleUrls: ['./scan-state-toolbar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-xl fx-row-vert-cent' },
})
export class ScanStateToolbarComponent extends StoreDispatcher implements OnInit {

  block: ScanStateBlock;
  formGroup: FormGroup;
  stream: boolean;
  openSidePanel: boolean;
  treeView: boolean;
  highlightSnarkPool: boolean;

  private inputRef: ElementRef<HTMLInputElement>;
  private gotHeightFromForm: boolean;

  @ViewChild('input') set content(c: ElementRef<HTMLInputElement>) {
    if (c) {
      this.inputRef = c;
      this.listenToInputChanges();
    }
  }

  constructor(private fb: FormBuilder) { super(); }

  ngOnInit(): void {
    this.initForm();
    this.listenToTreesChanges();
    this.listenToStreamChanges();
    this.listenToSidePanelChange();
    this.listenToTreeViewChange();
    this.listenToHighlightSnarkPoolChange();
  }

  getHeight(height: NumberInput): void {
    this.dispatch(ScanStateGetBlock, { heightOrHash: height });
    if (this.stream && height) {
      this.dispatch(ScanStatePause);
    } else if (!this.stream && !height) {
      this.dispatch(ScanStateStart);
    }
  }

  toggleSidePanel(): void {
    this.dispatch(ScanStateToggleSidePanel);
  }

  clearForm(): void {
    this.formGroup.get('search').setValue(null, { emitEvent: false });
  }

  private listenToTreesChanges(): void {
    this.select(selectScanStateBlock, (block: ScanStateBlock) => {
      this.block = block;
      if (![block?.hash, block?.height?.toString()].includes(this.formGroup.get('search').value) && this.gotHeightFromForm) {
        this.clearForm();
        this.gotHeightFromForm = false;
      }
      this.detect();
    });
  }

  private listenToTreeViewChange(): void {
    this.select(selectScanStateTreeView, tv => {
      this.treeView = tv;
      this.detect();
    });
  }

  private initForm(): void {
    this.formGroup = this.fb.group({
      search: [''],
    });
  }

  private listenToInputChanges(): void {
    fromEvent<KeyboardEvent>(this.inputRef.nativeElement, 'keyup')
      .pipe(
        untilDestroyed(this),
        filter((event: KeyboardEvent) => event.key === 'Enter'),
      )
      .subscribe(() => {
        this.inputRef.nativeElement.blur();
      });

    fromEvent(this.inputRef.nativeElement, 'focusout').pipe(
      untilDestroyed(this),
      debounceTime(400),
      map(() => this.formGroup.get('search').value),
      distinctUntilChanged(),
      filter((value: string) => !!value?.length),
    ).subscribe((value: string) => {
      this.gotHeightFromForm = true;
      this.getHeight(value.trim());
    });
  }

  toggleTreeView(tree: boolean): void {
    if (tree === this.treeView) {
      return;
    }
    this.dispatch(ScanStateToggleTreeView);
  }

  goLive(): void {
    this.dispatch(ScanStateStart);
  }

  pause(): void {
    this.dispatch(ScanStatePause);
  }

  private listenToStreamChanges(): void {
    this.select(selectScanStateStream, (stream: boolean) => {
      this.stream = stream;
      this.detect();
    });
  }

  private listenToSidePanelChange(): void {
    this.select(selectScanStateOpenSidePanel, open => {
      if (open && !this.openSidePanel) {
        this.openSidePanel = true;
        this.detect();
      } else if (!open && this.openSidePanel) {
        this.openSidePanel = false;
        this.detect();
      }
    }, mergeMap((open: boolean) => of(open).pipe(delay(open ? 0 : 250))));
  }

  private listenToHighlightSnarkPoolChange(): void {
    this.select(selectScanStateHighlightSnarkPool, highlight => {
      this.highlightSnarkPool = highlight;
      this.detect();
    });
  }

  toggleHighlightSnarkPool(): void {
    this.dispatch(ScanStateHighlightSnarkPool);
  }
}
