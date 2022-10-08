export function generateURL(root: string, directory: string, filename: string) {
  if (!directory) {
    return `${root}/${encodeURIComponent(filename)}`;
  } else {
    return `${root}/${directory
      .split("/")
      .map((component) => encodeURIComponent(component))
      .join("/")}/${encodeURIComponent(filename)}`;
  }
}
