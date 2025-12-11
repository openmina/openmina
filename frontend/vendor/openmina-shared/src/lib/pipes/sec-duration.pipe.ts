import { Pipe, PipeTransform } from '@angular/core';
import { MICROSEC_IN_1_SEC, MILLISEC_IN_1_SEC, NANOSEC_IN_1_SEC } from '../constants/unit-measurements';
import { formatNumber } from '@angular/common';
import { hasValue } from '../helpers/values.helper';

interface SecDurationConfigDefinition {
  severe: number;
  warn: number;
  default: number;
  color: boolean;
  onlySeconds: boolean;
  undefinedAlternative: string | number | undefined;
  includeMinutes: boolean;
  /**
   * If true, include minutes is also mandatory!
   */
  includeHours: boolean;
  colors: [string, string, string];
  valueIsZeroFn: () => string;
}

export type SecDurationConfig = Partial<SecDurationConfigDefinition>;
export const SEC_CONFIG_DEFAULT_PALETTE: [string, string, string] = ['warn', 'orange', 'error'];
export const SEC_CONFIG_GRAY_PALETTE: [string, string, string] = ['tertiary', 'secondary', 'primary'];

const baseConfig: SecDurationConfig = {
  severe: 1,
  warn: 0.3,
  default: 0.1,
  color: false,
  onlySeconds: false,
  includeMinutes: false,
  includeHours: false,
  colors: SEC_CONFIG_DEFAULT_PALETTE,
  valueIsZeroFn: null,
};

@Pipe({
    name: 'secDuration',
    standalone: false
})
export class SecDurationPipe implements PipeTransform {

  /**
   * @param value - number in seconds
   * @param config - configuration object
   * */
  transform(value: number, config: SecDurationConfig = baseConfig): string | number {
    let response;

    if (!value) {
      if (value === 0) {
        if (config.valueIsZeroFn) {
          return config.valueIsZeroFn();
        }
        return `${value}s`;
      } else {
        return config.undefinedAlternative !== undefined ? config.undefinedAlternative : value;
      }
    }

    if ((value >= 1 && !config.includeMinutes) || (config.includeMinutes && value < 60 && value >= 1) || config.onlySeconds || value < 0) {
      response = SecDurationPipe.format(value) + 's';
    } else if (value >= 60 && config.includeMinutes) {
      if (config.includeHours && value >= 3600) {
        response = formatNumber(value / 3600, 'en-US', '1.0-0') + 'h';
      } else {
        response = formatNumber(value / 60, 'en-US', '1.0-0') + 'm';
      }
    } else if (value >= 0.001) {
      response = SecDurationPipe.format(value * MILLISEC_IN_1_SEC) + 'ms';
    } else if (value >= 0.000001) {
      response = SecDurationPipe.format(value * MICROSEC_IN_1_SEC) + 'μs';
    } else {
      response = SecDurationPipe.format(value * NANOSEC_IN_1_SEC) + 'ns';
    }

    if (!config.color) {
      return response;
    } else if (!hasValue(config.colors)) {
      config.colors = SEC_CONFIG_DEFAULT_PALETTE;
    }

    if (value >= config.severe) {
      return `<span class="${config.colors[2]}">${response}</span>`;
    } else if (value >= config.warn) {
      return `<span class="${config.colors[1]}">${response}</span>`;
    } else if (value >= config.default) {
      return `<span class="${config.colors[0]}">${response}</span>`;
    }
    return response;
  }

  private static format(value: number): string {
    return formatNumber(value, 'en-US', '1.0-2');
  }

}
