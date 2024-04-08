export interface BlockProductionOverviewWindow {
  start: number;
  end: number;
  canonical: number;
  missed: number;
  orphaned: number;
  futureRights: number;
  interval: [number, number];
}
