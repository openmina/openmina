export const removeParamsFromURL = (path: string): string => {
  if (path?.includes('?')) {
    return path.split('?')[0];
  }
  return path;
};
