export interface BlockProductionWonSlotsEpoch {
  epochNumber: number;
  start: number;
  end: number;
  currentGlobalSlot: number;
  currentTime: number;
  vrfStats: {
    evaluated: number;
    total: number;
  };
}
