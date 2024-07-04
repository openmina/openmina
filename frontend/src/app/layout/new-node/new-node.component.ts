import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  EventEmitter,
  HostListener,
  OnInit,
  ViewChild,
} from '@angular/core';
import { StoreDispatcher } from '@shared/base-classes/store-dispatcher.class';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { FeaturesConfig, FeatureType, MinaNode } from '@shared/types/core/environment/mina-env.type';
import { AppActions } from '@app/app.actions';

const nodeNames: string[] = [
  'Crypto Hash',
  'Block Forge',
  'Ledger Node',
  'Protocol Hub',
  'Chain Matrix',
  'Cipher Node',
  'Crypto Core',
  'Token Vault',
  'Crypto Mesh',
  'Protocol Forge',
];
const servers: string[] = [
  '192.168.1.100:80',
  '10.0.0.1:443',
  '172.16.0.1:8080',
  '198.51.100.1:3000',
  '203.0.113.10:8000',
  '192.0.2.15:9000',
  '169.254.0.1:27017',
  '127.0.0.1:5432',
];


type Feature = {
  name: string;
  checked?: boolean;
  subFeatures: SubFeature[];
};

type SubFeature = {
  name: string;
  checked?: boolean;
};

@Component({
  selector: 'mina-new-node',
  templateUrl: './new-node.component.html',
  styleUrls: ['./new-node.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'flex-column bg-surface-top popup-box-shadow-weak border-rad-8' },
})
export class NewNodeComponent extends StoreDispatcher implements OnInit {

  protected readonly features: Feature[] = [
    {
      name: 'Dashboard',
      checked: true,
      subFeatures: [],
    },
    {
      name: 'Nodes',
      checked: true,
      subFeatures: [
        { name: 'overview', checked: true },
        { name: 'live', checked: true },
        { name: 'bootstrap', checked: true },
      ],
    },
    {
      name: 'State',
      checked: true,
      subFeatures: [
        { name: 'actions', checked: true },
      ],
    },
    {
      name: 'Network',
      checked: true,
      subFeatures: [
        { name: 'messages', checked: true },
        { name: 'connections', checked: true },
        { name: 'blocks', checked: true },
        { name: 'topology', checked: true },
        { name: 'node DHT', checked: true },
        { name: 'graph overview', checked: true },
        { name: 'bootstrap stats', checked: true },
      ],
    },
    {
      name: 'Snarks',
      checked: true,
      subFeatures: [
        { name: 'scan state', checked: true },
      ],
    },
    {
      name: 'Resources',
      checked: true,
      subFeatures: [
        { name: 'memory', checked: true },
      ],
    },
    {
      name: 'Mempool',
      checked: true,
      subFeatures: [],
    },
    {
      name: 'Benchmarks',
      checked: true,
      subFeatures: [],
    },
  ];
  readonly closeEmitter: EventEmitter<void> = new EventEmitter<void>();
  readonly placeholder: string = nodeNames[Math.floor(Math.random() * nodeNames.length)];
  readonly placeholder2: string = servers[Math.floor(Math.random() * servers.length)];
  readonly placeholder3: string = servers[Math.floor(Math.random() * servers.length)];
  readonly placeholder4: string = servers[Math.floor(Math.random() * servers.length)];
  formGroup: FormGroup;

  @ViewChild('fg') private fgRef: ElementRef<HTMLFormElement>;

  @HostListener('document:keydown', ['$event'])
  handleKeyboardEvent(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      this.close();
    }
  }

  @HostListener('document:click', ['$event'])
  clickOut(event: MouseEvent): void {
    if (!this.el.nativeElement.contains(event.target)) {
      this.close();
    }
  }

  constructor(private formBuilder: FormBuilder,
              private el: ElementRef) { super(); }

  ngOnInit(): void {
    this.initForm();
  }

  private initForm(): void {
    this.formGroup = this.formBuilder.group({
      name: ['', Validators.required],
      url: ['', Validators.required],
      memProfiler: [''],
      debugger: [''],
    });
  }

  onFeatureCheck(feature: Feature): void {
    feature.checked = !feature.checked;
    feature.subFeatures.forEach(subFeature => {
      subFeature.checked = feature.checked;
    });
  }

  onSubFeatureCheck(subFeature: SubFeature, feature: Feature): void {
    subFeature.checked = !subFeature.checked;
    feature.checked = feature.subFeatures.some(subFeature => subFeature.checked);
  }

  addNode(): void {
    if (this.formGroup.invalid) {
      this.fgRef.nativeElement.scroll({ top: 0, behavior: 'smooth' });
      this.formGroup.markAllAsTouched();
      return;
    }

    const featuresConfig: FeaturesConfig = {};
    this.features
      .filter(f => f.checked)
      .forEach((feature: Feature) => {
        const featureType = feature.name.toLowerCase().replace(' ', '-') as FeatureType;
        featuresConfig[featureType] = feature.subFeatures
          .filter(f => f.checked)
          .map((subFeature: SubFeature) => subFeature.name.toLowerCase().replace(' ', '-'));
      });

    const node: MinaNode = {
      name: this.formGroup.get('name').value,
      url: this.formGroup.get('url').value,
      memoryProfiler: this.formGroup.get('memProfiler').value,
      debugger: this.formGroup.get('debugger').value,
      features: featuresConfig,
      isCustom: true,
    };

    this.dispatch2(AppActions.addNode({ node }));
    this.close();
  }

  close(): void {
    this.closeEmitter.emit();
  }
}
