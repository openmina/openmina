import { ChangeDetectionStrategy, Component, EventEmitter, Output } from '@angular/core';
import { WebNodeService } from '@core/services/web-node.service';
import * as JSZip from 'jszip';
import { ManualDetection, OpenminaSharedModule } from '@openmina/shared';
import { animate, style, transition, trigger } from '@angular/animations';
import { CONFIG } from '@shared/constants/config';

@Component({
  selector: 'mina-web-node-file-upload',
  standalone: true,
  imports: [
    OpenminaSharedModule,
  ],
  templateUrl: './web-node-file-upload.component.html',
  styleUrl: './web-node-file-upload.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: { class: 'h-100 w-100 flex-column align-center' },
  animations: [
    trigger('messageChange', [
      transition(':enter', [
        style({ opacity: 0, transform: 'translateY(-10px)' }),
        animate('300ms ease-out', style({ opacity: 1, transform: 'translateY(0)' })),
      ]),
    ]),
  ],
})
export class WebNodeFileUploadComponent extends ManualDetection {

  @Output() startWebNode: EventEmitter<void> = new EventEmitter<void>();
  validFiles: boolean = false;
  error: boolean = false;
  uploadedFileName: string;

  constructor(private webnodeService: WebNodeService) { super(); }

  startCustomWebNode(): void {
    this.startWebNode.emit();
  }

  onStartDevelopWebnodeNonBP(): void {
    this.webnodeService.privateStake = null;
    this.webnodeService.noBlockProduction = true;
    delete CONFIG.globalConfig.features['block-production'];
    this.startWebNode.emit();
  }

  onStartDevelopWebnode(): void {
    this.webnodeService.privateStake = null;
    this.startWebNode.emit();
  }

  onFileSelected(event: any): void {
    this.processZipFile(event.target.files[0]).then(files => {
      const publicKey = files.find(f => f.name.includes('.pub'))?.content;
      const password = files.find(f => f.name.includes('password'))?.content.replace(/\r?\n|\r/g, '');
      const stake = files.find(f => f.name.includes('stake') && !f.name.includes('.pub'))?.content;
      if (this.error || !publicKey || !stake) {
        this.error = true;
      } else {
        this.webnodeService.privateStake = { publicKey, password, stake: JSON.parse(stake) };
        this.validFiles = true;
      }
      this.detect();
    });
  }

  private async processZipFile(zipFile: File): Promise<{ name: string, content: string }[]> {
    const fileContents: { name: string, content: string }[] = [];
    this.uploadedFileName = zipFile.name;

    try {
      const zip = await JSZip.loadAsync(zipFile);
      await Promise.all(Object.keys(zip.files).map(async (name) => {
        if (!zip.files[name].dir) { // Skip directories
          try {
            const content = await zip.files[name].async('string'); // Read file as text
            fileContents.push({ name, content });
          } catch (readError) {
            console.error(`Error reading file ${name}:`, readError);
            this.error = true;
          }
        }
      }));
    } catch (error) {
      console.error('Error processing ZIP file:', error);
      this.error = true;
    }
    return fileContents;
  }

  clearFiles(): void {
    this.validFiles = false;
    this.uploadedFileName = null;
    this.webnodeService.privateStake = null;
    this.error = false;
  }
}
