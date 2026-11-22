// The painted camp and ruin vignettes are transparent miniatures. Keep their supplied 512 px
// masters and derivatives together, and load only the size selected for the current zoom.
const urls = import.meta.glob<string>('../assets/map-features/*.png', {
  eager: true, query: '?url', import: 'default',
});
const images = new Map<string, HTMLImageElement>();
let onLoad: (() => void) | null = null;

export type MapFeature = 'camp' | 'ruin';

export function onMapFeatureLoad(cb: () => void) {
  onLoad = cb;
}

/** The authored far mark carries the identity when detailed miniatures no longer read. */
export function mapFeatureImage(kind: MapFeature, far: boolean, visible: boolean, hexPixels: number): HTMLImageElement | null {
  const subject = far ? `mark-${kind}` : `b-${kind}`;
  const remembered = visible ? '' : '-remembered';
  const size = far ? 64 : hexPixels >= 42 ? 512 : hexPixels >= 22 ? 128 : 64;
  const name = `${subject}${remembered}-${size}.png`;
  let image = images.get(name);
  if (!image) {
    const url = urls[`../assets/map-features/${name}`];
    if (!url || typeof Image === 'undefined') return null;
    image = new Image();
    image.onload = () => onLoad?.();
    image.src = url;
    images.set(name, image);
    return null;
  }
  return image.complete && image.naturalWidth > 0 ? image : null;
}
