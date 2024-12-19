import { FuzzingLine } from './fuzzing-line.type';

export interface FuzzingFileDetails {
  filename: string;
  lines: FuzzingLine[];
  executedLines: number;
}
