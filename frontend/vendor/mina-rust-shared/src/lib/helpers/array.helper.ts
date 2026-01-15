import { SortDirection, TableSort } from '../types/shared/table-sort.type';

export function sort<T = any>(inpArray: T[], sort: TableSort<T>, strings: Array<keyof T>, sortNulls: boolean = false): T[] {
  const sortProperty = sort.sortBy;
  const isStringSorting = strings.includes(sortProperty);
  const array: T[] = [...inpArray];

  let toBeSorted: T[];
  let toNotBeSorted: T[] = [];
  if (sortNulls) {
    toBeSorted = array;
  } else {
    toBeSorted = isStringSorting ? array : array.filter(e => e[sortProperty] !== undefined && e[sortProperty] !== null);
    toNotBeSorted = isStringSorting ? [] : array.filter(e => e[sortProperty] === undefined || e[sortProperty] === null);
  }

  if (isStringSorting) {
    const stringSort = (o1: T, o2: T) => {
      const s2 = (o2[sortProperty] || '') as string;
      const s1 = (o1[sortProperty] || '') as string;
      return sort.sortDirection === SortDirection.DSC
        ? (s2).localeCompare(s1)
        : s1.localeCompare(s2);
    };
    toBeSorted.sort(stringSort);
  } else {
    const numberSort = (o1: T, o2: T): number => {
      const o2Sort = (o2[sortProperty] ?? Number.MAX_VALUE) as number;
      const o1Sort = (o1[sortProperty] ?? Number.MAX_VALUE) as number;
      return sort.sortDirection === SortDirection.DSC
        ? o2Sort - o1Sort
        : o1Sort - o2Sort;
    };
    toBeSorted.sort(numberSort);
  }

  return [...toBeSorted, ...toNotBeSorted];
}

export function lastItem<T = any>(array: T[]): T {
  return array[array.length - 1];
}

export function toggleItem<T>(array: T[], item: T, comparator: (curr: T) => boolean = (curr: T) => curr === item): T[] {
  const index = array.findIndex(comparator);
  if (index !== -1) {
    array = [...array.slice(0, index), ...array.slice(index + 1)];
  } else {
    array = [...array, item];
  }
  return array;
}

export function removeLast<T>(arr: T[]): T[] {
  return arr.slice(0, arr.length - 1);
}
