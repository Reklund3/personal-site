import { useEffect } from 'react';
import routes from '../seo/routes.json';

/**
 * Client-side SEO.
 *
 * The server (src/routes/home/mod.rs) is the single source of truth for every
 * crawler-facing tag: it injects title, description, canonical, OpenGraph,
 * Twitter, and JSON-LD into index.html per route at the `<!--SEO-->` marker.
 * Crawlers and social unfurlers fetch the raw HTML and never run our SPA
 * navigation, so that server-injected set is what they read.
 *
 * The one thing that must still change on client-side navigation is the browser
 * tab title. We set document.title imperatively rather than rendering a <title>
 * (and other <meta>/<link>) element. This matters because the app mounts with
 * createRoot, not hydrateRoot: React has no knowledge of the server-rendered
 * tags, so any tag it renders is *appended*, producing duplicate
 * <title>/<meta>/<link>/JSON-LD nodes and a second <link rel="canonical"> built
 * from window.location.origin that conflicts with the server's (config-derived)
 * one. Mutating the existing <title> text sidesteps both problems.
 */

interface RouteMetadata {
  title: string;
  description: string;
  keywords?: string;
  ogType?: 'website' | 'profile' | 'article';
  includeProfileTags?: boolean;
}

const routesMap: Record<string, RouteMetadata> = routes as Record<string, RouteMetadata>;

/**
 * Keep the browser tab title in sync with the active route during client-side
 * navigation. Renders no DOM — meta/OG/canonical/JSON-LD all come from the
 * server-injected HTML.
 */
export function SEOMetaTags({ path }: { path: string }) {
  const meta = routesMap[path] || routesMap['/'];
  const fullTitle = `${meta.title} | Robert Eklund`;

  useEffect(() => {
    document.title = fullTitle;
  }, [fullTitle]);

  return null;
}
