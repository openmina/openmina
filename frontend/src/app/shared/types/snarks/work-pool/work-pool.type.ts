import { WorkPoolCommitment } from '@shared/types/snarks/work-pool/work-pool-commitment.type';
import { WorkPoolSnark } from '@shared/types/snarks/work-pool/work-pool-snark.type';

export interface WorkPool {
  datetime: string;
  timestamp: number;
  id: string;
  commitment: WorkPoolCommitment;
  snark: WorkPoolSnark;
  snarkRecLatency: number;
  snarkOrigin: 'Local' | 'Remote';
  commitmentRecLatency: number;
  commitmentOrigin: 'Local' | 'Remote';
  commitmentCreatedLatency: number;
  notSameCommitter: boolean;
}
