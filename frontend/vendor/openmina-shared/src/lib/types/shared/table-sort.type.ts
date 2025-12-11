export interface TableSort<T> {
  sortBy: keyof T;
  sortDirection: SortDirection.ASC | SortDirection.DSC;
}

export enum SortDirection {
  ASC = 'ascending',
  DSC = 'descending'
}
