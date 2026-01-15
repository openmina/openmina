export const getXTicks = (series: any[], ticksCount: number, xProperty: string): string[] => {
  const xTicks = [];
  const delta = Math.floor(series.length / ticksCount);
  for (let i = 0; i <= series.length; i = i + delta) {
    if (series[i]) {
      xTicks.push(series[i][xProperty]);
    }
  }
  return xTicks;
};

/**
 * Instantiates a new instance of the NiceScale class.
 * Calculate and update values for tick spacing and nice
 * minimum and maximum data points on the axis.
 */
export function niceYScale(min: number, max: number, maxTicks: number): [tickSpacing: number, niceMax: number, niceMin: number] {
  const minPoint = min;
  const maxPoint = max;
  const range = niceNum(maxPoint - minPoint, false);
  const tickSpacing = niceNum(range / (maxTicks - 1), true);
  const niceMin = Math.floor(minPoint / tickSpacing) * tickSpacing;
  const niceMax = Math.ceil(maxPoint / tickSpacing) * tickSpacing;
  return [tickSpacing, niceMax, niceMin];
}

/**
 * Returns a "nice" number approximately equal to range Rounds
 * the number if round = true Takes the ceiling if round = false.
 *  localRange the data range
 *  round whether to round the result
 *  a "nice" number to be used for the data range
 */
function niceNum(localRange: number, round: boolean): number {
  /** exponent of localRange */
  let exponent: number;
  /** fractional part of localRange */
  let fraction: number;
  /** nice, rounded fraction */
  let niceFraction: number;

  exponent = Math.floor(Math.log10(localRange));
  fraction = localRange / Math.pow(10, exponent);

  if (round) {
    if (fraction < 1.5)
      niceFraction = 1;
    else if (fraction < 3)
      niceFraction = 2;
    else if (fraction < 7)
      niceFraction = 5;
    else
      niceFraction = 10;
  } else {
    if (fraction <= 1)
      niceFraction = 1;
    else if (fraction <= 2)
      niceFraction = 2;
    else if (fraction <= 5)
      niceFraction = 5;
    else
      niceFraction = 10;
  }

  return niceFraction * Math.pow(10, exponent);
}
