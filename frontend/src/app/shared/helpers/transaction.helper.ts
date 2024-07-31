import { toReadableDate } from '@openmina/shared';

export function removeUnicodeEscapes(str: string): string {
  let cleaned = str;
  cleaned = cleaned.replace(/\\u[0-9a-fA-F]{4}/g, '');
  cleaned = cleaned.replace(/[^a-zA-Z0-9,\.]/g, '');
  return cleaned;
}

export function getTimeFromMemo(memo: string): string {
  return memo?.includes(',') ? toReadableDate(memo.split(',')[0].replace('S.T.', '')) : undefined;
}
