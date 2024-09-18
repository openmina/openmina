import { toReadableDate } from '@openmina/shared';
import bs58check from 'bs58check';

export function decodeMemo(memo: string): string {
  try {
    const buffer = bs58check.decode(memo);

    // The first byte is the version, we can skip it
    const payload = buffer.slice(1);

    // Convert the payload to a string
    const tag = payload[0];
    const len = payload[1];
    let result = '';
    if (tag === 0) {
      result = toHexString(payload.slice(0));
    } else {
      for (let i = 2; i < len + 2; i++) {
        result += String.fromCharCode(payload[i]);
      }
    }

    return result;
  } catch (error) {
    console.error('Error decoding memo:', error);
    throw error;
  }
}

function toHexString(byteArray: Uint8Array) {
  return Array.from(byteArray, (byte) => {
    return ('0' + (byte & 0xFF).toString(16)).slice(-2);
  }).join('');
}

export function removeUnicodeEscapes(str: string): string {
  let cleaned = str;
  cleaned = cleaned.replace(/\\u[0-9a-fA-F]{4}/g, '');
  cleaned = cleaned.replace(/[^a-zA-Z0-9,\.]/g, '');
  return cleaned;
}

export function getTimeFromMemo(memo: string): string {
  return memo?.includes(',') ? toReadableDate(memo.split(',')[0].replace('S.T.', '')) : undefined;
}
