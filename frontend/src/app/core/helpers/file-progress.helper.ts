import { BehaviorSubject } from 'rxjs';
import { safelyExecuteInBrowser } from '@openmina/shared';

class AssetMonitor {
  readonly downloads: Map<string, any> = new Map();
  readonly progress$: BehaviorSubject<any>;

  constructor(progress$: BehaviorSubject<any>) {
    this.progress$ = progress$;
    safelyExecuteInBrowser(() => {
      this.setupInterceptor();
    });
  }

  private setupInterceptor(): void {
    const originalFetch = window.fetch;
    const self = this;

    window.fetch = async function (resource, options) {
      // Only intercept asset requests (you can modify these extensions as needed)
      const assetExtensions = ['.wasm'];
      const isAsset = assetExtensions.some(ext =>
        resource.toString().toLowerCase().endsWith(ext),
      );

      if (!isAsset) {
        return originalFetch(resource, options);
      }

      const startTime = performance.now();
      const downloadInfo = {
        url: resource.toString(),
        startTime,
        progress: 0,
        totalSize: 0,
        status: 'pending',
        endTime: 0,
        duration: 0,
      };

      self.downloads.set(resource.toString(), downloadInfo);
      self.emitProgress(downloadInfo);

      try {
        const response = await originalFetch(resource, options);
        const reader = response.clone().body.getReader();
        const contentLength = +response.headers.get('Content-Length');
        downloadInfo.totalSize = contentLength;
        let receivedLength = 0;

        while (true) {
          try {
            const { done, value } = await reader.read();

            if (done) {
              break;
            }

            receivedLength += value.length;
            downloadInfo.progress = (receivedLength / contentLength) * 100;
            self.emitProgress(downloadInfo);
          } catch (error) {
            downloadInfo.status = 'error';
            self.emitProgress(downloadInfo);
            throw error;
          }
        }

        downloadInfo.status = 'complete';
        downloadInfo.endTime = performance.now();
        downloadInfo.duration = downloadInfo.endTime - downloadInfo.startTime;
        self.emitProgress(downloadInfo);
        return await response;
      } catch (error_1) {
        downloadInfo.status = 'error';
        self.emitProgress(downloadInfo);
        throw error_1;
      }
    };
  }

  private emitProgress(downloadInfo: any): void {
    this.progress$.next({
      url: downloadInfo.url,
      progress: downloadInfo.progress.toFixed(2),
      totalSize: downloadInfo.totalSize,
      status: downloadInfo.status,
      duration: downloadInfo.duration,
      startTime: downloadInfo.startTime,
      endTime: downloadInfo.endTime,
      downloaded: downloadInfo.progress * downloadInfo.totalSize / 100,
    });
  }
}

export class FileProgressHelper {
  static progress$: BehaviorSubject<any> = new BehaviorSubject<any>(null);

  static initDownloadProgress(): void {
    new AssetMonitor(this.progress$);
  }
}
