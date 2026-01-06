export function isStringAValidJson(str: string): boolean {
  try {
    JSON.parse(str);
    return true;
  } catch (e) {
    return false;
  }
}

export function downloadJson(toDownload: object | string, fileName: string): void {
  const jsonString = typeof toDownload === 'object' ? JSON.stringify(toDownload) : toDownload;
  const href = 'data:text/json;charset=UTF-8,' + encodeURIComponent(jsonString);
  downloadFromAnchorElement(href, fileName);
}

export function downloadJsonFromURL(url: string, fileName: string, cancelCallback: () => boolean, element?: HTMLElement): void {
  if (element) {
    element.textContent = 'Saving ...';
  }
  fetch(url)
    .then(response => response.blob())
    .then((blob: Blob) => {
      if (!cancelCallback()) {
        const blobURL = URL.createObjectURL(blob);
        downloadFromAnchorElement(blobURL, fileName);
      }
      if (element) {
        element.textContent = 'Save JSON';
      }
    });
}

function downloadFromAnchorElement(href: string, fileName: string): void {
  const element: HTMLAnchorElement = document.createElement('a');
  element.href = href;
  element.classList.add('d-none');
  element.download = fileName;
  document.body.appendChild(element);
  element.click();
  document.body.removeChild(element);
}
