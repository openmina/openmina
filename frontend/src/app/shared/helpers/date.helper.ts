import { ONE_THOUSAND } from '@openmina/shared';

/**
 * Get the difference between the current time and the given time
 * Example: getTimeDiff(1626825600000) => { diff: '1d 1h 30m', inFuture: true }
 * @param time
 * @param config - withSecs: boolean, fromTime: number
 */
export function getTimeDiff(time: number, config?: { withSecs: boolean, fromTime?: number }): {
  diff: string,
  inFuture: boolean
} {
  if (time.toString().length === 10) {
    time *= ONE_THOUSAND;
  }

  const paramTime = new Date(time).getTime();
  const currentTime = config?.fromTime ? new Date(config.fromTime).getTime() : Date.now();

  let timeDifference = paramTime - currentTime;
  const inFuture = timeDifference > 0;
  timeDifference = Math.abs(timeDifference);

  const days = Math.floor(timeDifference / (1000 * 60 * 60 * 24));
  timeDifference -= days * 1000 * 60 * 60 * 24;

  const hours = Math.floor(timeDifference / (1000 * 60 * 60));
  timeDifference -= hours * 1000 * 60 * 60;

  const minutes = Math.floor(timeDifference / (1000 * 60));
  timeDifference -= minutes * 1000 * 60;

  const seconds = Math.floor(timeDifference / 1000);

  let timeAgo = '';
  if (days > 0) {
    timeAgo += `${days}d `;
  }
  if (hours > 0) {
    timeAgo += `${hours}h `;
  }
  if (days === 0 && hours !== 0 && minutes > 0) {
    timeAgo += `${minutes}m `;
  }
  if (days === 0 && hours === 0 && minutes > 0) {
    timeAgo += `${minutes}m `;
  }
  if (config?.withSecs && days === 0 && hours === 0) {
    timeAgo += `${seconds}s `;
  }
  if (!config?.withSecs && days === 0 && hours === 0 && minutes === 0) {
    timeAgo = '<1m ';
  }

  return { diff: timeAgo.trim(), inFuture };
}
