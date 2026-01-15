import { getWindow } from './browser.helper';

export const hasValue = (value: any): boolean => {
  return value !== undefined && value !== null;
}

export function any(arg: any): any {
  return arg as any;
}

export const isMobile = (): boolean => getWindow()?.innerWidth <= 768;
export const isDesktop = (): boolean => !isMobile();

export const nanOrElse = (value: number, replacement: number | string | null | undefined): any => {
  if (isNaN(value)) {
    return replacement;
  }
  return value;
}
