import { ChangeDetectionStrategy, Component, Inject, OnDestroy } from '@angular/core';
import { MinaTableRustWrapper } from '@shared/base-classes/mina-table-rust-wrapper.class';
import { TableColumnList, TooltipPosition } from '@openmina/shared';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import { selectMemoryResourcesActiveResource } from '@resources/memory/memory-resources.state';
import { MemoryResourcesSetActiveResource } from '@resources/memory/memory-resources.actions';
import { DOCUMENT } from '@angular/common';

@Component({
  selector: 'app-memory-resources-table',
  templateUrl: './memory-resources-table.component.html',
  styleUrls: ['./memory-resources-table.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column h-100 border-top' },
})
export class MemoryResourcesTableComponent extends MinaTableRustWrapper<MemoryResource> implements OnDestroy {

  activeResource: MemoryResource;
  tooltipWidth: number = Math.max(window.innerWidth - 40, 1500);
  position: TooltipPosition = TooltipPosition.RIGHT;

  protected readonly tableHeads: TableColumnList<MemoryResource> = [
    { name: 'children' },
    { name: 'share' },
    { name: 'executable' },
    { name: 'size' },
    { name: 'function' },
  ];
  private customTableStyleElement: HTMLStyleElement | HTMLElement;

  constructor(@Inject(DOCUMENT) private document: Document) {
    super();
  }


  override async ngOnInit(): Promise<void> {
    await super.ngOnInit();
    this.listenToActiveMemoryChanges();
  }

  protected override setupTable(): void {
    this.table.gridTemplateColumns = [70, 170, 140, 100, 'auto'];
    this.table.minWidth = 980;
  }

  private listenToActiveMemoryChanges(): void {
    this.select(selectMemoryResourcesActiveResource, (resource: MemoryResource) => {
      this.activeResource = resource;
      this.table.rows = resource?.children || [];
      this.addCustomStyleElement();
      this.table.detect();
    });
  }

  protected override onRowClick(row: MemoryResource): void {
    if (row.children.length === 0) {
      return;
    }
    this.dispatch(MemoryResourcesSetActiveResource, row);
  }

  private addCustomStyleElement(): void {
    this.customTableStyleElement = this.document.getElementById('memory-resources-custom-table') ?? this.document.createElement<'style'>('style');
    this.customTableStyleElement.id = 'memory-resources-custom-table';
    let indexesWithChildren: number[] = [];
    let indexesWithoutChildren: number[] = [];
    this.table.rows.forEach((curr: MemoryResource, i: number) => {
      if (curr.children.length > 0) {
        indexesWithChildren.push(i);
      } else {
        indexesWithoutChildren.push(i);
      }
    });

    this.customTableStyleElement.textContent = '';
    indexesWithChildren.forEach(i => {
      this.customTableStyleElement.textContent +=
        `app-memory-resources-table mina-table .row[idx="${i}"] {border-radius: 4px; background-color: var(--base-surface-top) !important;pointer-events: auto}`
        + `app-memory-resources-table mina-table .row[idx="${i}"]:hover {background-color: var(--base-tertiary2) !important}`;
    });
    indexesWithoutChildren.forEach(i => {
      this.customTableStyleElement.textContent +=
        `app-memory-resources-table mina-table .row[idx="${i}"]:hover:not(.active):not(.head) .secondary {color: var(--base-secondary) !important}`
        + `app-memory-resources-table mina-table .row[idx="${i}"]:hover:not(.active):not(.head) .perc {color: var(--base-secondary) !important}`
        + `app-memory-resources-table mina-table .row[idx="${i}"]:hover:not(.active):not(.head) .perc.yellow {color: var(--aware-primary) !important}`
        + `app-memory-resources-table mina-table .row[idx="${i}"]:hover:not(.active):not(.head) .perc.red {color: var(--warn-primary) !important}`
    });
    this.document.head.appendChild(this.customTableStyleElement);
  }

  override ngOnDestroy(): void {
    super.ngOnDestroy();
    this.customTableStyleElement?.remove();
  }
}
