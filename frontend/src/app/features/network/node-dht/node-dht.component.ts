import { ChangeDetectionStrategy, Component, ElementRef, OnInit } from '@angular/core';

@Component({
  selector: 'app-node-dht',
  templateUrl: './node-dht.component.html',
  styleUrls: ['./node-dht.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class NodeDhtComponent implements OnInit {

  constructor(public el: ElementRef<HTMLElement>) { }

  ngOnInit(): void {
  }
}
