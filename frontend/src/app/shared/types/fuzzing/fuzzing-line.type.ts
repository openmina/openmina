import { FuzzingLineCounter } from './fuzzing-line-counter.type';

export interface FuzzingLine {
  line: string;
  counters: FuzzingLineCounter[];
  lineColor: string;
  lineHits: number;
  html: string;
}
