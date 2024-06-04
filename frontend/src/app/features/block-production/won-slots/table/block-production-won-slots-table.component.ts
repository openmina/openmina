import { ChangeDetectionStrategy, Component, OnInit, TemplateRef, ViewChild } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { getMergedRoute, MergedRoute, SecDurationConfig, TableColumnList } from '@openmina/shared';
import { Router } from '@angular/router';
import { SnarksWorkPoolToggleSidePanel } from '@snarks/work-pool/snarks-work-pool.actions';
import { filter, take } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import {
  BlockProductionWonSlotsSlot, BlockProductionWonSlotsStatus,
} from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';
import { BlockProductionWonSlotsSelectors } from '@block-production/won-slots/block-production-won-slots.state';
import { BlockProductionWonSlotsActions } from '@block-production/won-slots/block-production-won-slots.actions';

@Component({
  selector: 'mina-block-production-won-slots-table',
  templateUrl: './block-production-won-slots-table.component.html',
  styleUrls: ['./block-production-won-slots-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column' },
})
export class BlockProductionWonSlotsTableComponent extends MinaTableRustWrapper<BlockProductionWonSlotsSlot> implements OnInit {

  protected readonly BlockProductionWonSlotsStatus = BlockProductionWonSlotsStatus;
  protected readonly secConfig: SecDurationConfig = {
    color: true,
    undefinedAlternative: '-',
    default: 100,
    warn: 500,
    severe: 1000,
  };
  protected readonly tableHeads: TableColumnList<BlockProductionWonSlotsSlot> = [
    { name: 'status', sort: 'message' },
    { name: 'age', sort: 'slotTime' },
    { name: 'height' },
    { name: 'global slot', sort: 'globalSlot' },
    { name: 'transactions', sort: 'transactionsTotal' },
    { name: 'snark fees', sort: 'snarkFees' },
    { name: 'coinbase reward', sort: 'coinbaseRewards' },
    { name: 'tx fees reward', sort: 'txFeesRewards' },
  ];

  openSidePanel: boolean;

  @ViewChild('thGroupsTemplate') private thGroupsTemplate: TemplateRef<void>;

  private fromRoute: string;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToActiveSlotChange();
    this.listenToNodesChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [210, 140, 110, 120, 120, 100, 150, 150];
    this.table.propertyForActiveCheck = 'globalSlot';
    this.table.thGroupsTemplate = this.thGroupsTemplate;
    this.table.sortAction = BlockProductionWonSlotsActions.sort;
    this.table.sortSelector = BlockProductionWonSlotsSelectors.sort;
    this.table.trackByFn = (_: number, item: BlockProductionWonSlotsSlot) => item.message + item.slotTime + item.transactionsTotal + item.snarkFees + item.coinbaseRewards + item.txFeesRewards;
  }

  toggleSidePanel(): void {
    this.dispatch(SnarksWorkPoolToggleSidePanel);
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['id'] && this.table.rows.length === 0) {
        this.fromRoute = route.params['id'];
      }
    }, take(1));
  }

  private listenToNodesChanges(): void {
    this.select(BlockProductionWonSlotsSelectors.filteredSlots, (slots: BlockProductionWonSlotsSlot[]) => {
      this.table.rows = slots;
      this.table.detect();
      if (this.fromRoute && slots.length > 0) {
        this.scrollToElement();
      }
      this.detect();
    }, filter((slots: BlockProductionWonSlotsSlot[]) => slots.length > 0));
  }

  private listenToActiveSlotChange(): void {
    this.select(BlockProductionWonSlotsSelectors.activeSlot, (slot: BlockProductionWonSlotsSlot) => {
      if (!this.table.activeRow) {
        this.fromRoute = slot.globalSlot.toString();
      }
      this.table.activeRow = slot;
      this.table.detect();
      this.detect();
    }, filter(Boolean));
  }

  private scrollToElement(): void {
    const finder = (node: BlockProductionWonSlotsSlot) => node.globalSlot.toString() === this.fromRoute;
    const i = this.table.rows.findIndex(finder);
    this.table.scrollToElement(finder);
    delete this.fromRoute;
    this.onRowClick(this.table.rows[i]);
  }

  protected override onRowClick(slot: BlockProductionWonSlotsSlot): void {
    if (!slot) {
      return;
    }
    if (this.table.activeRow?.globalSlot !== slot?.globalSlot) {
      this.dispatch2(BlockProductionWonSlotsActions.setActiveSlot({ slot }));
      this.router.navigate([Routes.BLOCK_PRODUCTION, Routes.WON_SLOTS, slot.globalSlot], { queryParamsHandling: 'merge' });
    }
  }
}
