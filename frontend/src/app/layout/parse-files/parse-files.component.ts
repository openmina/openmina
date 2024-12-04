import { ChangeDetectionStrategy, Component } from '@angular/core';
import { NgFor, NgIf } from '@angular/common';
import * as JSZip from 'jszip';

@Component({
  selector: 'mina-parse-files',
  standalone: true,
  imports: [
    NgIf, NgFor,
  ],
  templateUrl: './parse-files.component.html',
  styleUrl: './parse-files.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ParseFilesComponent {

  fileContents: string[] = [];

  onFileSelected(event: any) {
    const files: FileList = event.target.files;

    // Reset previous file contents
    this.fileContents = [];

    // Loop through selected files
    for (let i = 0; i < files.length; i++) {
      const file = files[i];

      // Ensure it's a .txt file
    }
    this.handleFileUpload(event);
  }

  async processZipFile(zipFile: File) {
    try {
      // Load the ZIP file
      const zip = await JSZip.loadAsync(zipFile);

      // Array to store file contents
      const fileContents: { name: string, content: string }[] = [];

      // Iterate through each file in the ZIP
      await Promise.all(Object.keys(zip.files).map(async (filename) => {
        // Skip directories
        if (!zip.files[filename].dir) {
          try {
            // Read file as text
            const content = await zip.files[filename].async('string');
            fileContents.push({
              name: filename,
              content: content,
            });
          } catch (readError) {
            console.error(`Error reading file ${filename}:`, readError);
          }
        }
      }));

      // Return or process the file contents
      return fileContents;
    } catch (error) {
      console.error('Error processing ZIP file:', error);
      return [];
    }
  }

// Usage example
  handleFileUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      const zipFile = input.files[0];
      this.processZipFile(zipFile).then(files => {
        files.forEach(file => {
          console.log(`File: ${file.name}`);
          console.log(`Content: ${file.content.substring(0, 200)}...`);
        });
      });
    }
  }

  private readFileContent(file: File) {
    const reader = new FileReader();

    reader.onload = (e: any) => {
      const content = e.target.result as string;
      this.fileContents.push(content);

      // Perform operations on the content here
      this.processFileContent(content);
    };

    reader.onerror = (e) => {
      console.error('Error reading file', e);
    };

    // Read the file as text
    reader.readAsText(file);
  }

  private processFileContent(content: string) {
    // Example operations
    const lines = content.split('\n');
    const wordCount = content.split(/\s+/).length;
    const characterCount = content.length;

    console.log('Lines:', lines);
    console.log('Word Count:', wordCount);
    console.log('Character Count:', characterCount);

    // Add your specific file processing logic here
  }
}
