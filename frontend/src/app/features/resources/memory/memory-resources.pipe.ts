import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'resSize',
})
export class ResourcesSizePipe implements PipeTransform {
  transform(kilobytes: number): string {
    if (kilobytes >= 1048576) {
      return `${(kilobytes / 1048576).toFixed(2)} GB`;
    } else if (kilobytes >= 1024) {
      return `${(kilobytes / 1024).toFixed(2)} MB`;
    }
    return `${kilobytes?.toFixed(2) || 0} KB`;
  }
}
