import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  Inject,
  TemplateRef,
  ViewChild,
} from '@angular/core';
import { CdkVirtualScrollViewport } from '@angular/cdk/scrolling';
import { untilDestroyed } from '@ngneat/until-destroy';
import { debounceTime } from 'rxjs';
import { CommonModule, DOCUMENT } from '@angular/common';
import { BaseStoreDispatcher } from '../../base-classes/base-store-dispatcher.class';
import { TableColumnList } from '../../types/shared/table-head-sorting.type';
import { SortDirection, TableSort } from '../../types/shared/table-sort.type';
import { hasValue, isMobile } from '../../helpers/values.helper';
import { OpenminaEagerSharedModule } from '../../openmina-eager-shared.module';
import { ActionCreator, Action } from '@ngrx/store';

const DESKTOP_ROW_HEIGHT = 36;

@Component({
    imports: [OpenminaEagerSharedModule, CommonModule],
    selector: 'mina-table',
    templateUrl: './mina-table.component.html',
    styleUrls: ['./mina-table.component.scss'],
    changeDetection: ChangeDetectionStrategy.OnPush,
    host: { class: 'h-100 flex-column' }
})
export class MinaTableComponent<T extends object> extends BaseStoreDispatcher<any> implements AfterViewInit {

  rowSize: number = DESKTOP_ROW_HEIGHT;
  isMobile: boolean;

  rows: T[] = [];
  activeRow: T;
  tableHeads: TableColumnList<T>;
  rowTemplate: TemplateRef<{ row: T, i: number }>;
  currentSort: TableSort<T>;
  thGroupsTemplate: TemplateRef<void>;
  propertyForActiveCheck: keyof T;
  gridTemplateColumns: Array<number | 'auto' | '1fr'> = [];
  minWidth: number;
  sortClz: new (payload: TableSort<T>) => { type: string, payload: TableSort<T> };
  sortAction: ActionCreator<string, (props: { sort: TableSort<T>; }) => { sort: TableSort<T>; } & Action<string>>;
  sortSelector: (state: any) => TableSort<T>;
  rowClickCallback: (row: T) => void;
  trackByFn: (index: number, row: T) => any = (index: number, row: T) => row;

  tableLevel: number = 1;

  @ViewChild(CdkVirtualScrollViewport) private vs: CdkVirtualScrollViewport;
  @ViewChild('toTop') private toTop: ElementRef<HTMLDivElement>;
  private hiddenToTop: boolean = true;

  constructor(@Inject(DOCUMENT) private document: Document,
              private el: ElementRef) { super(); }

  init(): void {
    this.minWidth = this.minWidth || this.gridTemplateColumns.reduce<number>((acc: number, curr: number | string) => acc + Number(curr), 0);
    this.addGridTemplateColumnsInCssFile();
    this.listenToSortingChanges();
    this.detect();
  }

  ngAfterViewInit(): void {
    this.listenToScrolling();
    this.positionToTop();
  }

  private addGridTemplateColumnsInCssFile(): void {
    let value = `mina-table #table${this.tableLevel}.mina-table .row{grid-template-columns:`;
    this.gridTemplateColumns.forEach(v => value += typeof v === 'number' ? `${v}px ` : `${v} `);
    this.document.getElementById('table-style' + this.tableLevel).textContent = value + '}';
  }

  sortTable(sortBy: string | keyof T): void {
    const sortDirection = sortBy !== this.currentSort.sortBy
      ? this.currentSort.sortDirection
      : this.currentSort.sortDirection === SortDirection.ASC ? SortDirection.DSC : SortDirection.ASC;
    const sort = { sortBy: sortBy as keyof T, sortDirection };
    if (this.sortClz) {
      this.dispatch(this.sortClz, sort);
    } else if (this.sortAction) {
      this.dispatch2(this.sortAction({ sort }));
    }
  }

  scrollToTop(): void {
    this.vs.scrollToIndex(0, 'smooth');
    this.toTop.nativeElement.classList.add('hide');
    this.hiddenToTop = true;
  }

  scrollToElement(rowFinder: (row: T) => boolean): void {
    const topElements = Math.round(this.vs.elementRef.nativeElement.offsetHeight / 2 / this.rowSize) - 3;
    const jobIndex = this.rows.findIndex(rowFinder);
    this.vs.scrollToIndex(jobIndex - topElements);
  }

  get virtualScroll(): CdkVirtualScrollViewport {
    return this.vs;
  }

  private listenToScrolling(): void {
    this.vs.scrolledIndexChange
      .pipe(debounceTime(this.hiddenToTop ? 200 : 0), untilDestroyed(this))
      .subscribe(index => {
        if (index === 0) {
          this.toTop.nativeElement.classList.add('hide');
        } else {
          this.toTop.nativeElement.classList.remove('hide');
        }
        this.hiddenToTop = index === 0;
      });
  }

  private listenToSortingChanges(): void {
    if (!this.sortSelector) return;
    this.select(this.sortSelector, (sort: TableSort<T>) => {
      this.currentSort = sort;
      this.detect();
    });
  }

  checkViewport(isMobile: boolean): void {
    if (this.isMobile !== isMobile) {
      this.isMobile = isMobile;
      this.rowSize = isMobile ? (26 * this.tableHeads.length + 10) : DESKTOP_ROW_HEIGHT;
      this.vs?.checkViewportSize();
      this.detect();
    }
  }

  private positionToTop(): void {
    if (!isMobile()) {
      return;
    }
    const rect = this.el.nativeElement.getBoundingClientRect();

    this.toTop.nativeElement.style.top = `${rect.top + rect.height - 60}px`;
    this.toTop.nativeElement.style.left = `${rect.left + rect.width - 60}px`;
  }

  onVsClick(event: MouseEvent): void {
    let target = event.target as any;
    let idx: number = null;
    while (target && target.getAttribute) {
      let attrValue = target.getAttribute('idx');
      if (attrValue) {
        attrValue = Number(attrValue);
        idx = attrValue;
      }
      if (hasValue(idx) || target === this.vs.elementRef.nativeElement) {
        break;
      }
      target = target.parentElement;
    }

    if (hasValue(idx)) {
      this.rowClickCallback(this.rows[idx]);
    }
  }
}
