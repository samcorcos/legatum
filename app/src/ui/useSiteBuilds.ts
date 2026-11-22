import { useEffect, useState } from 'react';
import { fetchVersions, type SiteBuild } from '../platform/versions';

/** The builds the website serves (platform/versions.ts), read when the screen opens; null until then or without a list. */
export function useSiteBuilds(): SiteBuild[] | null {
  const [builds, setBuilds] = useState<SiteBuild[] | null>(null);
  useEffect(() => {
    let live = true;
    void fetchVersions().then((list) => { if (live) setBuilds(list); });
    return () => { live = false; };
  }, []);
  return builds;
}
