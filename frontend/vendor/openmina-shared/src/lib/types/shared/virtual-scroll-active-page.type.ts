export class VirtualScrollActivePage<T> {
  id?: number;
  numberOfRecords?: number;
  start?: T;
  end?: T;
  firstPageIdWithFilters?: number;
  lastPageIdWithFilters?: number;
  firstPageIdWithTimestamp?: number;
  lastPageIdWithTimestamp?: number; //this has value only when you hit directly the last page
}
