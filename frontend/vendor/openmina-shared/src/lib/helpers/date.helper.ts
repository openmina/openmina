import { formatDate } from '@angular/common';

function convertToReadableDate(value: number | string, format: string = 'HH:mm:ss.SSS, dd MMM yy'): string {
  return formatDate(value, format, 'en-US');
}

export const toReadableDate = (value: number | string, format?: string): string => convertToReadableDate(value, format);
export const noMillisFormat = 'HH:mm:ss, dd MMM yy';
