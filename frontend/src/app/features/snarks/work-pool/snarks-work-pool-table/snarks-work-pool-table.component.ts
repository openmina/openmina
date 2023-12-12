import { ChangeDetectionStrategy, Component, OnInit, TemplateRef, ViewChild } from '@angular/core';
import { getMergedRoute, MergedRoute, SecDurationConfig, TableColumnList } from '@openmina/shared';
import { Router } from '@angular/router';
import { take } from 'rxjs';
import { Routes } from '@shared/enums/routes.enum';
import { WorkPool } from '@shared/types/snarks/work-pool/work-pool.type';
import {
  SnarksWorkPoolSetActiveWorkPool,
  SnarksWorkPoolSortWorkPool,
  SnarksWorkPoolToggleSidePanel
} from '@snarks/work-pool/snarks-work-pool.actions';
import {
  selectSnarksWorkPoolActiveWorkPool,
  selectSnarksWorkPoolOpenSidePanel,
  selectSnarksWorkPools,
  selectSnarksWorkPoolSort
} from '@snarks/work-pool/snarks-work-pool.state';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';

@Component({
  selector: 'mina-snarks-work-pool-table',
  templateUrl: './snarks-work-pool-table.component.html',
  styleUrls: ['./snarks-work-pool-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SnarksWorkPoolTableComponent extends MinaTableRustWrapper<WorkPool> implements OnInit {

  readonly secConfig: SecDurationConfig = {
    color: true,
    undefinedAlternative: '-',
    default: 100,
    warn: 500,
    severe: 1000
  };
  protected readonly tableHeads: TableColumnList<WorkPool> = [
    { name: 'datetime', sort: 'timestamp' },
    { name: 'id' },
    { name: 'status', sort: 'commitment' },
    { name: 'created latency', sort: 'commitmentCreatedLatency' },
    { name: 'received latency', sort: 'commitmentRecLatency' },
    { name: 'origin', sort: 'commitmentOrigin' },
    { name: 'status', sort: 'snark' },
    { name: 'received latency', sort: 'snarkRecLatency' },
    { name: 'origin', sort: 'snarkOrigin' },
  ];

  openSidePanel: boolean;

  @ViewChild('thGroupsTemplate') private thGroupsTemplate: TemplateRef<void>;

  private wpFromRoute: string;

  constructor(private router: Router) { super(); }

  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToRouteChange();
    this.listenToNodesChanges();
    this.listenToActiveNodeChange();
    this.listenToSidePanelToggling();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [165, 140, 110, 150, 150, 100, 110, 150, 100];
    this.table.propertyForActiveCheck = 'id';
    this.table.thGroupsTemplate = this.thGroupsTemplate;
    this.table.sortClz = SnarksWorkPoolSortWorkPool;
    this.table.sortSelector = selectSnarksWorkPoolSort;
  }

  toggleSidePanel(): void {
    this.dispatch(SnarksWorkPoolToggleSidePanel);
  }

  private listenToRouteChange(): void {
    this.select(getMergedRoute, (route: MergedRoute) => {
      if (route.params['id'] && this.table.rows.length === 0) {
        this.wpFromRoute = route.params['id'];
      }
    }, take(1));
  }

  private listenToNodesChanges(): void {
    this.select(selectSnarksWorkPools, (wp: WorkPool[]) => {
      this.table.rows = wp;
      this.table.detect();
      if (this.wpFromRoute && wp.length > 0) {
        this.scrollToElement();
      }
      this.detect();
    });
  }

  private listenToActiveNodeChange(): void {
    this.select(selectSnarksWorkPoolActiveWorkPool, (activeWp: WorkPool) => {
      this.table.activeRow = activeWp;
      this.table.detect();
      this.detect();
    });
  }

  private scrollToElement(): void {
    const finder = (node: WorkPool) => node.id === this.wpFromRoute;
    const i = this.table.rows.findIndex(finder);
    this.table.scrollToElement(finder);
    delete this.wpFromRoute;
    this.onRowClick(this.table.rows[i]);
  }

  protected override onRowClick(row: WorkPool): void {
    if (!row) {
      return;
    }
    if (this.table.activeRow?.id !== row?.id) {
      this.dispatch(SnarksWorkPoolSetActiveWorkPool, { id: row.id });
      this.router.navigate([Routes.SNARKS, Routes.WORK_POOL, row.id], { queryParamsHandling: 'merge' });
    }
  }

  private listenToSidePanelToggling(): void {
    this.select(selectSnarksWorkPoolOpenSidePanel, (open: boolean) => {
      this.openSidePanel = open;
      this.detect();
    });
  }
}
