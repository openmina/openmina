import { Pipe, PipeTransform } from '@angular/core';

const KB_FACTOR = 1024;
const MB_FACTOR = 1048576;
const GB_FACTOR = 1073741824;

@Pipe({
    name: 'size',
    standalone: false
})
export class SizePipe implements PipeTransform {

  transform(value: number, wrapInHTML: boolean = true): string {
    if (value < 1000) {
      return value + ' B';
    } else if (value < 1000000) {
      const kb = value / KB_FACTOR;
      const response = `${SizePipe.addDecimals(kb)} KB`;
      return wrapInHTML ? `<span class="warn">${response}</span>` : response;
    } else if (value < 1000000000) {
      const mb = value / MB_FACTOR;
      const response = `${SizePipe.addDecimals(mb)} MB`;
      return wrapInHTML ? `<span class="error">${response}</span>` : response;
    } else {
      const gb = value / GB_FACTOR;
      const response = `${SizePipe.addDecimals(gb)} GB`;
      return wrapInHTML ? `<span class="error">${response}</span>` : response;
    }
  }

  private static addDecimals(num: number): string {
    const decimals = num >= 10 ? (num >= 100 ? 0 : 1) : 2;
    return num.toFixed(decimals);
  }
}
