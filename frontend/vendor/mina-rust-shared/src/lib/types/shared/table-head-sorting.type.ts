interface TableHeadSorting<T = any> {
  name: string | keyof T;
  sort?: keyof T;
  tooltip?: string | number;
}

export type TableColumnList<T = object> = TableHeadSorting<T>[];
