import { ChangeDetectionStrategy, Component, Inject, OnInit } from '@angular/core';
import { StateActionsToggleSidePanel } from '@state/actions/state-actions.actions';
import {
  selectStateActionsActiveSlotAndStats,
  selectStateActionsGroups,
  selectStateActionsSort
} from '@state/actions/state-actions.state';
import { StateActionGroup } from '@shared/types/state/actions/state-action-group.type';
import { isMobile, SecDurationConfig, TableColumnList, TableSort } from '@openmina/shared';
import { DOCUMENT } from '@angular/common';
import { distinctUntilChanged } from 'rxjs';
import { StateActionsStats } from '@shared/types/state/actions/state-actions-stats.type';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-state-actions-side-panel',
  templateUrl: './state-actions-side-panel.component.html',
  styleUrls: ['./state-actions-side-panel.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StateActionsSidePanelComponent extends MinaTableRustWrapper<StateActionGroup> implements OnInit {

  activeSlot: number;
  stats: StateActionsStats = {} as StateActionsStats;
  readonly secConfig: SecDurationConfig = { onlySeconds: true, undefinedAlternative: '-' };

  protected readonly tableHeads: TableColumnList<StateActionGroup> = [
    { name: 'category' },
    { name: 'calls', sort: 'count' },
    { name: 'mean', sort: 'meanTime' },
    { name: 'total', sort: 'totalTime' },
  ];

  constructor(@Inject(DOCUMENT) private document: Document) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToSlotChange();
    this.listenToActionsChange();
    this.listenToSort();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = ['auto', 60, 80, 105];
    this.table.minWidth = 350;
  }

  closeSidePanel(): void {
    this.dispatch(StateActionsToggleSidePanel);
  }

  private listenToSlotChange(): void {
    this.select(selectStateActionsActiveSlotAndStats, ([activeSlot, stats]: [number, StateActionsStats]) => {
      this.activeSlot = activeSlot;
      this.stats = stats;
      this.detect();
    });
  }

  private listenToActionsChange(): void {
    this.select(selectStateActionsGroups, (groups: StateActionGroup[]) => {
      this.table.rows = groups.filter(g => g.display);
      this.table.detect();
      this.detect();
    });
  }

  private listenToSort(): void {
    if (isMobile()) return;
    const primary = 'primary';
    const tableHeads = this.document.querySelectorAll('mina-table .row.head span');
    this.select(selectStateActionsSort, (sort: TableSort<StateActionGroup>) => {
      const activeSortColumnIndex = this.tableHeads.findIndex(h => h.sort === sort.sortBy);
      tableHeads.forEach(span => span.classList.remove(primary));
      tableHeads.item(activeSortColumnIndex).classList.add(primary);
    }, distinctUntilChanged((curr, prev) => curr.sortBy === prev.sortBy));
  }
}
