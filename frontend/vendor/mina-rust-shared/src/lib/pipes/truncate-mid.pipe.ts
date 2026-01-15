import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
    name: 'truncateMid',
    standalone: false
})
export class TruncateMidPipe implements PipeTransform {

  transform(value: string, firstSlice: number = 6, secondSlice: number = 6): string {
    if (!value) {
      return '';
    }
    return value.length > (firstSlice + secondSlice) ? value.slice(0, firstSlice) + '...' + value.slice(value.length - secondSlice) : value;
  }

}
