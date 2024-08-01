import { ChangeDetectionStrategy, Component, ElementRef, OnInit } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { ScanStateTree } from '@shared/types/snarks/scan-state/scan-state-tree.type';
import {
  selectScanStateActiveLeaf,
  selectScanStateBlock, selectScanStateHighlightSnarkPool,
  selectScanStateOpenSidePanel,
} from '@snarks/scan-state/scan-state.state';
import { ScanStateBlock } from '@shared/types/snarks/scan-state/scan-state-block.type';
import { delay, filter, mergeMap, of, skip, take, tap } from 'rxjs';
import { ScanStateGetBlock, ScanStateStart, ScanStateToggleSidePanel } from '@snarks/scan-state/scan-state.actions';
import { ScanStateLeaf } from '@shared/types/snarks/scan-state/scan-state-leaf.type';
import { getMergedRoute, MergedRoute, isMobile } from '@openmina/shared';

@Component({
  selector: 'mina-scan-state-tree-list',
  templateUrl: './scan-state-tree-list.component.html',
  styleUrls: ['./scan-state-tree-list.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-minus-xl flex-column p-relative' },
})
export class ScanStateTreeListComponent extends StoreDispatcher implements OnInit {

  trees: ScanStateTree[] = [];
  height: number;
  error: boolean;
  activeLeaf: ScanStateLeaf;
  openSidePanel: boolean;
  isMobile: boolean = isMobile();
  highlightSnarks: boolean;

  private activeJobIdInRoute: string;

  readonly trackTree = (_: number, tree: ScanStateTree): string => {
    return ''
      + tree.ongoing
      + tree.availableJobs
      + tree.completedSnarks
      + tree.empty
      + tree.coinbase
      + tree.payment
      + tree.zkApp
      + tree.feeTransfer
      + tree.merge;
  };

  constructor(private el: ElementRef<HTMLElement>) {super();}

  ngOnInit(): void {
    this.listenToHighlightSnarkPoolChange();
    this.listenToTreesChanges();
    this.listenToActiveJobID();
    this.listenToRoute();
    if (this.isMobile) {
      this.listenToSidePanelChange();
    }
  }

  private listenToTreesChanges(): void {
    this.select(selectScanStateBlock, (block: ScanStateBlock) => {
      this.height = block.height;
      this.trees = block.trees;
      this.error = block.trees.length === 0;
      this.detect();
    }, filter(block => !!block));
  }

  checkLatestHeight(): void {
    this.dispatch(ScanStateGetBlock, { heightOrHash: null });
    this.dispatch(ScanStateStart);
  }

  private listenToActiveJobID(): void {
    this.select(selectScanStateActiveLeaf, (leaf: ScanStateLeaf) => {
      this.activeLeaf = leaf;
      if (this.activeLeaf?.scrolling) {
        setTimeout(() => {
          this.el.nativeElement
            .querySelectorAll(`div.flex-column`)
            .item(this.activeLeaf.treeIndex)
            .scrollIntoView({ behavior: 'smooth', block: 'start' });
        }, 50);
      }
      delete this.activeJobIdInRoute;
      this.detect();
    });
  }

  private listenToRoute(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      this.activeJobIdInRoute = route.queryParams['jobId'];
    }, take(1));
  }

  toggleSidePanel(): void {
    this.dispatch(ScanStateToggleSidePanel);
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
    this.select(selectScanStateHighlightSnarkPool, (highlight: boolean) => {
      this.detect();
    }, tap(h => this.highlightSnarks = h), skip(1));
  }
}
