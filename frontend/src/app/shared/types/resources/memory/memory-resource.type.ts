import { MemoryResourceName } from '@shared/types/resources/memory/memory-resources-name.type';

export class MemoryResource {
  name: MemoryResourceName;
  children: MemoryResource[];
  value?: number;
  id: number;
}
