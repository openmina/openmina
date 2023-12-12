import { Injectable } from '@angular/core';
import { MemoryResourceName } from '@shared/types/resources/memory/memory-resources-name.type';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import { map, Observable, of } from 'rxjs';
import { HttpClient } from '@angular/common/http';
import { memoryResourcesTreeMapMock } from '@resources/memory/mock';

@Injectable({
  providedIn: 'root'
})
export class MemoryResourcesService {

  constructor(private http: HttpClient) { }

  getStorageResources(reversed: boolean = false, threshold: number = 512): Observable<MemoryResource> {
    // return this.http.get<MemoryResourceTree>(`${api}/v1/tree?threshold=${threshold}&reverse=${reversed}`)
    return of(memoryResourcesTreeMapMock)
      .pipe(map((response: MemoryResourceTree) => this.mapMemoryResponse(response, threshold)));
  }

  private mapMemoryResponse(response: MemoryResourceTree, threshold: number): MemoryResource {
    return {
      name: { ...response.name, executableName: 'root' },
      value2: round(response.value - (response.cacheValue || 0)),
      children: this.build(response.frames, threshold)
    };
  }

  private build(frames: MemoryResourceTree[], threshold: number): MemoryResource[] {
    const children: MemoryResource[] = [];
    frames
      .forEach(frame => {
        const size = round(frame.value - (frame.cacheValue || 0));
        const items: MemoryResource = {
          name: this.getFrameName(frame.name, threshold),
          value2: size,
          children: this.build(frame.frames || [], threshold),
          color: this.appendColorForFrame(size)
        };
        children.push(items);
      });
    return children.sort((c1: MemoryResource, c2: MemoryResource) => c2.value2 - c1.value2);
  }

  private getFrameName(name: MemoryResourceTree['name'], threshold: number): MemoryResourceName {
    if (typeof name === 'string') {
      return {
        executableName: name === 'underThreshold' ? `below ${round(threshold)} mb` : name,
        functionName: null,
      };
    }

    return {
      executableName: name.executable + '@' + name.offset,
      functionName: name.functionName
    };
  }

  private appendColorForFrame(value: number): string {
    if (value > 99.99) {
      return '#793541';
    } else if (value > 49.99) {
      return '#555558';
    } else if (value > 9.99) {
      return '#323248';
    } else {
      return '#386038';
    }
  }
}

const round = (num: number): number => +(Math.round(Number((num / 1024) + 'e+2')) + 'e-2');

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

/*

  private init(data: MemoryResource): void {
    this.width = this.treeMapRef.nativeElement.offsetWidth;
    this.height = this.treeMapRef.nativeElement.offsetHeight;
    this.margin = { top: 2, right: 2, bottom: 2, left: 2 };

    this.svg = d3.select(this.treeMapRef.nativeElement).append('svg')
      .attr('width', this.width)
      .attr('height', this.height);

    const innerWidth = this.width - this.margin.left - this.margin.right;
    const innerHeight = this.height - this.margin.top - this.margin.bottom;

    const root: MemoryResource = {
      name: data.name,
      children: data.children,
    };

    const treemap = d3.treemap<MemoryResource>()
      .size([innerWidth, innerHeight])
      .padding(1)
      .round(true);

    const rootNode = d3.hierarchy(root)
      .sum(d => d.value || 0)
      .sort((a, b) => b.value! - a.value!)
      .eachBefore(d => { if (d.depth > 1) d.children = null; }); // Filter out children beyond level 1

    treemap(rootNode);

    this.rectGroup = this.svg.append('g')
      .attr('class', 'nodes')
      .attr('transform', `translate(${rootNode.x0}, ${rootNode.y0})`);

    this.textGroup = this.svg.append('g')
      .attr('class', 'texts')
      .attr('transform', `translate(${rootNode.x0}, ${rootNode.y0})`);

    this.currentData = rootNode; // Initially set the currentData to rootNode

    const rects = this.rectGroup.selectAll('rect')
      .data(rootNode.children || []);

    rects.enter()
      .append('rect')
      .merge(rects)
      .attr('x', d => d.x0 - rootNode.x0)
      .attr('y', d => d.y0 - rootNode.y0)
      .attr('width', d => d.x1 - d.x0)
      .attr('height', d => d.y1 - d.y0)
      .attr('fill', d => d.data.color)
      .on('click', ($ev, d) => this.zoomIn(d));

    const texts = this.textGroup.selectAll('text')
      .data(rootNode.children || []);

    texts.enter()
      .append('text')
      .merge(texts)
      .attr('x', d => (d.x0 + d.x1) / 2 - rootNode.x0)
      .attr('y', d => (d.y0 + d.y1) / 2 - rootNode.y0)
      .attr('dy', '0.35em')
      .attr('text-anchor', 'middle')
      .attr('fill', 'white')
      .text(d => d.data.name.executableName)
      .each((d, i, nodes) => {
        const textWidth = nodes[i].getComputedTextLength();
        const rectWidth = d.x1 - d.x0;
        if (textWidth > rectWidth) {
          d3.select(nodes[i]).remove();
        }
      });
  }

  private zoomIn(clickedData: MemoryResource): void {
    if (!clickedData.children) {
      return;
    }

    const innerWidth = this.width - this.margin.left - this.margin.right;
    const innerHeight = this.height - this.margin.top - this.margin.bottom;

    const treemap = d3.treemap<MemoryResource>()
      .size([innerWidth, innerHeight])
      .padding(1)
      .round(true);

    const rootNode = d3.hierarchy(clickedData)
      .sum(d => d.value || 0)
      .sort((a, b) => b.value! - a.value!).eachBefore(d => { if (d.depth > 1) d.children = null; }); // Filter out children beyond level 1

    treemap(rootNode);

    this.rectGroup
      .selectAll('rect')
      .data(rootNode.descendants().slice(1)) // Select only the descendants of the clicked element
      .join('rect')
      .attr('x', d => d.x0 - rootNode.x0)
      .attr('y', d => d.y0 - rootNode.y0)
      .attr('width', d => d.x1 - d.x0)
      .attr('height', d => d.y1 - d.y0)
      .attr('fill', d => d.data.color)
      .on('click', ($ev, d) => this.zoomIn(d));

    this.textGroup
      .selectAll('text')
      .data(rootNode.descendants().slice(1))
      .join('text')
      .attr('x', d => (d.x0 + d.x1) / 2 - rootNode.x0)
      .attr('y', d => (d.y0 + d.y1) / 2 - rootNode.y0)
      .attr('dy', '0.35em')
      .attr('text-anchor', 'middle')
      .attr('fill', 'white')
      .text(d => {
        return d.data.data.name.executableName
      })
      .each((d, i, nodes) => {
        const textWidth = nodes[i].getComputedTextLength();
        const rectWidth = d.x1 - d.x0;
        if (textWidth > rectWidth) {
          d3.select(nodes[i]).remove();
        }
      });

    this.currentData = rootNode; // Update currentData to the new zoomed-in data
  }
  */
