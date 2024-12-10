import { Injectable } from '@angular/core';
import { delay, map, Observable } from 'rxjs';
import { FuzzingFile } from '@shared/types/fuzzing/fuzzing-file.type';
import { HttpClient } from '@angular/common/http';
import { FuzzingFileDetails } from '@shared/types/fuzzing/fuzzing-file-details.type';
import { FuzzingLineCounter } from '@shared/types/fuzzing/fuzzing-line-counter.type';
import { CONFIG } from '@shared/constants/config';
import { FuzzingDirectory } from '@shared/types/fuzzing/fuzzing-directory.type';
import { noMillisFormat, toReadableDate } from '@openmina/shared';

@Injectable({ providedIn: 'root' })
export class FuzzingService {

  constructor(private http: HttpClient) { }

  getRootDirectoryContent(): Observable<FuzzingDirectory[]> {
    // const url = CONFIG.server.includes(origin)
    //   ? 'assets/reports/index.json'
    //   : `${CONFIG.server}?path=${encodeURIComponent(CONFIG.parentDirectoryAbsolutePath)}`;
    const url = 'assets/reports/index.json';
    return this.http.get<string[]>(url).pipe(
      delay(50),
      map((dirNames: string[]) => this.mapGetDirectoryNamesResponse(dirNames)),
    );
  }

  private mapGetDirectoryNamesResponse(dirNames: string[]): FuzzingDirectory[] {
    const directories = dirNames.map((directory: string) => {
      let date: string;
      let dateNumber: number;

      try {
        date = toReadableDate(directory.split('_')[0], noMillisFormat);
        dateNumber = new Date(date).getTime();
      } catch (e) {
        date = '-';
        dateNumber = 0;
      }
      return {
        fullName: directory,
        name: directory.includes('_') ? directory.split('_').slice(1).join('_') : directory,
        date,
        dateNumber,
      };
    });
    return directories.sort((a: FuzzingDirectory, b: FuzzingDirectory) => b.dateNumber - a.dateNumber);
  }

  getFiles(activeDir: string): Observable<FuzzingFile[]> {
    // const url = CONFIG.server.includes(origin)
    //   ? `assets/reports/${activeDir}/${type}index.json`
    //   : `${CONFIG.server}/${type}index.json?path=${encodeURIComponent(CONFIG.parentDirectoryAbsolutePath + '/' + activeDir)}`;
    const url = `assets/reports/${activeDir}/rustindex.json`;
    return this.http.get<any[]>(url).pipe(delay(100))
      .pipe(
        map((files: any[]) => files.map((file: any) => ({
          name: file[0],
          coverage: file[1],
          path: file[2],
        }))),
      );
  }

  getFileDetails(activeDir: string, name: string): Observable<FuzzingFileDetails> {
    // const url = CONFIG.server.includes(origin)
    //   ? `assets/reports/${activeDir}/${name}`
    //   : `${CONFIG.server}/${name}?path=${encodeURIComponent(CONFIG.parentDirectoryAbsolutePath + '/' + activeDir)}`;
    const url = `assets/reports/${activeDir}/${name}`;
    return this.http.get<any>(url).pipe(delay(100))
      .pipe(
        map((file: any) => ({
          filename: file.filename,
          executedLines: file.lines.filter((line: any) => line.counters[0]).length,
          lines: file.lines.map((line: any) => {
            const counters = line.counters.map((counter: any) => ({
              colStart: counter.col_start,
              colEnd: counter.col_end,
              count: Math.abs(counter.count),
            }));
            return {
              line: line.line,
              lineColor: this.getLineColor(line),
              html: this.colorLineCounters(line.line, counters),
              lineHits: counters.length ? Math.max(...counters.map((counter: any) => counter.count)) : undefined,
              counters,
            };
          }),
        } as FuzzingFileDetails)),
      );
  }

  private getLineColor(line: any): string {
    if (line.counters.length === 0) {
      return line;
    }

    let lineColor = 'aware';

    for (const counter of line.counters) {
      if (counter.count > 0) {
        if (lineColor === 'warn') {
          lineColor = 'aware';
          break;
        }
        lineColor = 'success';
      } else {
        if (lineColor === 'success') {
          lineColor = 'aware';
          break;
        }
        lineColor = 'warn';
      }
    }
    return lineColor;
  }

  private colorLineCounters(line: string, counters: FuzzingLineCounter[]): string {
    let result = '';
    if (counters.length === 0) {
      return line.replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    for (let i = 0; i < line.length; i++) {
      const column = i;
      const c = line.charAt(i);
      const counter = counters.find((counter: FuzzingLineCounter) => counter.colStart <= column && counter.colEnd >= column);

      if (counter && column === counter.colStart) {
        const colorCode: string = `var(--${counter.count === 0 ? 'warn' : 'success'}-secondary)`;
        result += `<span style="color:var(--base-primary);background:${colorCode}" h="${counter.count}">`;
      }

      result += c === ' ' ? '&nbsp;' : (c === '<' ? '&lt;' : c === '>' ? '&gt;' : c);

      if (counter && column === counter.colEnd) {
        result += '</span>';
      }
    }

    return result;
  }
}
