import { Directive, OnInit, TemplateRef, ViewChild, ViewContainerRef } from '@angular/core';
import { MinaTableComponent } from '../components/mina-table/mina-table.component';
import { BaseStoreDispatcher, TableColumnList } from '@openmina/shared';

@Directive()
export abstract class MinaTableWrapper<T extends object, State> extends BaseStoreDispatcher<State> implements OnInit {

  protected abstract readonly tableHeads: TableColumnList<T>;

  @ViewChild('rowTemplate') protected rowTemplate: TemplateRef<{ row: T, i: number }>;
  @ViewChild('minaTable', { read: ViewContainerRef }) protected containerRef: ViewContainerRef;

  public table: MinaTableComponent<T>;

  async ngOnInit(): Promise<void> {
    await import('../components/mina-table/mina-table.component').then(c => {
      this.table = this.containerRef.createComponent(c.MinaTableComponent<T>).instance;
      this.table.tableHeads = this.tableHeads;
      this.table.rowTemplate = this.rowTemplate;
      this.table.rowClickCallback = (row: T, isRealClick: boolean = true) => this.onRowClick(row, isRealClick);
      this.setupTable();
      this.table.init();
    });
  }

  protected checkViewport(isMobile: boolean): void {
    this.table.checkViewport(isMobile);
  }

  protected abstract setupTable(): void;

  protected onRowClick(row: T, isRealClick: boolean): void { }
}
