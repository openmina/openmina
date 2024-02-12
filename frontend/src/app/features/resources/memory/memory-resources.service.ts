import { Injectable } from '@angular/core';
import { MemoryResourceName } from '@shared/types/resources/memory/memory-resources-name.type';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import { map, Observable } from 'rxjs';
import { RustService } from '@core/services/rust.service';

@Injectable({
  providedIn: 'root',
})
export class MemoryResourcesService {

  private id: number = 0;

  constructor(private rust: RustService) { }

  getStorageResources(threshold: number, reversed: boolean = false): Observable<MemoryResource> {
    this.id = 0;
    return this.rust.getMemProfiler<MemoryResourceTree>(`/v1/tree?threshold=${threshold}&reverse=${reversed}`)
      .pipe(map((response: MemoryResourceTree) => this.mapMemoryResponse(response, threshold)));
  }

  private mapMemoryResponse(response: MemoryResourceTree, threshold: number): MemoryResource {
    return {
      name: { ...response.name, executableName: 'root' },
      value: round(response.value/* - (response.cacheValue || 0)*/),
      children: this.build(response.frames, threshold),
      id: this.nextId,
    };
  }

  private build(frames: MemoryResourceTree[], threshold: number): MemoryResource[] {
    const children: MemoryResource[] = [];
    frames
      .forEach(frame => {
        const size = round(frame.value/* - (frame.cacheValue || 0)*/);
        const items: MemoryResource = {
          name: this.getFrameName(frame.name, threshold),
          value: size,
          children: this.build(frame.frames || [], threshold),
          id: this.nextId,
        };
        children.push(items);
      });
    return children.sort((c1: MemoryResource, c2: MemoryResource) => c2.value - c1.value);
  }

  private getFrameName(name: MemoryResourceTree['name'], threshold: number): MemoryResourceName {
    if (typeof name === 'string') {
      return {
        executableName: name === 'underThreshold' ? `below ${round(threshold)} KB` : name,
        functionName: null,
      };
    }

    return {
      executableName: `${name.executable}@${name.offset}`,
      functionName: name.functionName,
    };
  }

  private get nextId(): number {
    return this.id++;
  }
}

const round = (num: number): number => +(Math.round(Number(num + 'e+2')) + 'e-2');

interface MemoryResourceTree {
  name: {
    offset: string;
    executable: string;
    functionName: string;
    functionCategory: string;
  };
  value: number;
  cacheValue: number;
  frames: MemoryResourceTree[];
}
