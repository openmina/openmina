import { Pipe, PipeTransform } from '@angular/core';
import { ONE_MILLION } from '../constants/unit-measurements';
import { toReadableDate } from '../helpers/date.helper';

@Pipe({
    name: 'readableDate',
    standalone: false
})
export class ReadableDatePipe implements PipeTransform {
  transform(value: number, format?: string): string {
    if (value > 1e13) {
      value = value / ONE_MILLION;
    }

    return value ? toReadableDate(value, format) : '-';
  }
}
