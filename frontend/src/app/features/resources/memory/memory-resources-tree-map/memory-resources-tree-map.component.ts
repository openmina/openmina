//@ts-nocheck
import { AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, NgZone, ViewChild } from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { MemoryResourcesService } from '@resources/memory/memory-resources.service';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';
import * as d3 from 'd3';

@Component({
  selector: 'app-memory-resources-tree-map',
  templateUrl: './memory-resources-tree-map.component.html',
  styleUrls: ['./memory-resources-tree-map.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MemoryResourcesTreeMapComponent extends StoreDispatcher implements AfterViewInit {

  @ViewChild('treeMapChart') private treeMapRef: ElementRef<HTMLDivElement>;

  private width: number;
  private height: number;
  private margin = { top: 2, right: 2, bottom: 2, left: 2 };
  private svg: d3.Selection<SVGSVGElement, unknown, null, undefined>;

  constructor(private memoryService: MemoryResourcesService,
              private zone: NgZone) { super(); }

  ngAfterViewInit(): void {
    this.memoryService.getStorageResources().subscribe(data => {
      console.log(data);
      const mockData: MemoryResource = {
        name: { executableName: 'Root' },
        value2: 90,
        children: [
          {
            name: { executableName: 'Category1' },
            value2: 20,
            color: '#ff0000',
            children: [
              {
                name: { executableName: 'Subcategory1' },
                value2: 15,
                color: '#ff6666',
              },
              {
                name: { executableName: 'Subcategory2' },
                value2: 5,
                color: '#ff9999',
              },
            ],
          },
          {
            name: { executableName: 'Category2' },
            value2: 30,
            color: '#7f562b',
            children: [
              {
                name: { executableName: 'Subcategory3' },
                value2: 12,
                color: '#cc9966',
              },
              {
                name: { executableName: 'Subcategory4' },
                value2: 18,
                color: '#ffcc99',
              },
            ],
          },
          {
            name: { executableName: 'Category3' },
            value2: 40,
            color: '#0000ff',
            children: [
              {
                name: { executableName: 'Subcategory5' },
                value2: 28,
                color: '#6666ff',
              },
              {
                name: { executableName: 'Subcategory6' },
                value2: 12,
                color: '#9999ff',
              },
            ],
          },
          // Add more child categories as needed
        ],
      };
      this.zone.runOutsideAngular(() => this.createTreemapChart(data));
    });
  }

  private createTreemapChart(data: MemoryResource): void {
    const tile = (node, x0, y0, x1, y1) => {
      d3.treemapBinary(node, 0, 0, this.width, this.height);
      for (const child of node.children) {
        child.x0 = x0 + (child.x0 / this.width) * (x1 - x0);
        child.x1 = x0 + (child.x1 / this.width) * (x1 - x0);
        child.y0 = y0 + (child.y0 / this.height) * (y1 - y0);
        child.y1 = y0 + (child.y1 / this.height) * (y1 - y0);
      }
    };

    const position = (group, root) => {
      group.selectAll('g')
        .attr('transform', (d) => d === root ? `translate(0,-30)` : `translate (${x(d.x0)},${y(d.y0)})`)
        .select('rect')
        .attr('width', (d) => d === root ? this.width : x(d.x1) - x(d.x0))
        .attr('height', (d) => d === root ? 30 : y(d.y1) - y(d.y0));
    };

    const render = (group, root) => {
      const node = group
        .selectAll('g')
        .data(root.children.concat(root))
        .join('g');

      node.filter((d) => d === root ? d.parent : d.children)
        .attr('cursor', 'pointer')
        .on('click', (event, d) => d === root ? zoomout(root) : zoomin(d));

      node.append('rect')
        .attr('id', (d) => (d.leafUid = `leaf-${randomNum()}`).id)
        .attr('fill', d => d.data.color);

      node.append('text')
        .attr('font-weight', (d) => d === root ? 'bold' : null)
        .selectAll('tspan')
        .data((d) => {
          return d.data.name.executableName
            .split(/(?=[A-Z][^A-Z])/g)
            .concat('real: ' + format(d.data.value2))
            .concat('computed: ' + format(d.value))
        })
        .join('tspan')
        .attr('fill', '#fff')
        .attr('x', 3)
        .attr('y', (d, i, nodes) => `${(i === nodes.length - 1) * 0.3 + 1.1 + i * 0.9}em`)
        .attr('fill-opacity', (d, i, nodes) => i === nodes.length - 1 ? 0.7 : null)
        .attr('font-weight', (d, i, nodes) => i === nodes.length - 1 ? 'normal' : null)
        .text((d) => d);

      group.call(position, root);
    };

    this.width = this.treeMapRef.nativeElement.offsetWidth - this.margin.left - this.margin.right;
    this.height = this.treeMapRef.nativeElement.offsetHeight - this.margin.top - this.margin.bottom;

    const hierarchy = d3.hierarchy(data)
      .sum((d) => d.value2)
      .sort((a, b) => {
        return b.value - a.value;
      });
    const root = d3.treemap().tile(tile)(hierarchy);

    const x = d3.scaleLinear().rangeRound([0, this.width]);
    const y = d3.scaleLinear().rangeRound([0, this.height]);

    const format = d3.format(',d');

    this.svg = d3.select(this.treeMapRef.nativeElement)
      .append('svg')
      .attr('viewBox', [0.5, -30.5, this.width, this.height + 30])
      .attr('width', this.width)
      .attr('height', this.height + 30)
      .style('font', '10px');

    let group = this.svg.append('g')
      .call(render, root);

    const zoomin = (d) => {
      const group0 = group.attr('pointer-events', 'none');
      const group1 = (group = this.svg.append('g')).call(render, d);

      x.domain([d.x0, d.x1]);
      y.domain([d.y0, d.y1]);

      this.svg.transition()
        .duration(750)
        .call((t) => group0.transition(t).remove()
          .call(position, d.parent))
        .call((t) => group1.transition(t)
          .attrTween('opacity', () => d3.interpolate(0, 1))
          .call(position, d));
    };

    const zoomout = (d) => {
      const group0 = group.attr('pointer-events', 'none');
      const group1 = (group = this.svg.insert('g', '*')).call(render, d.parent);

      x.domain([d.parent.x0, d.parent.x1]);
      y.domain([d.parent.y0, d.parent.y1]);

      this.svg.transition()
        .duration(750)
        .call((t) => group0.transition(t).remove()
          .attrTween('opacity', () => d3.interpolate(1, 0))
          .call(position, d))
        .call((t) => group1.transition(t)
          .call(position, d.parent));
    };
  }
}

function randomNum() {
  return Math.floor(Math.random() * 10000) + 10000;
}
