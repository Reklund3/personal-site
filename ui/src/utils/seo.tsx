import React from 'react';
import routes from '../seo/routes.json';

/**
 * SEO metadata utilities for React 19 native metadata support
 */

interface RouteMetadata {
  title: string;
  description: string;
  keywords?: string;
  ogType?: 'website' | 'profile' | 'article';
  includeProfileTags?: boolean;
}

const routesMap: Record<string, RouteMetadata> = routes as Record<string, RouteMetadata>;

interface SEOMetaTagsProps {
  path: string;
  title?: string;
  description?: string;
  keywords?: string;
  ogType?: 'website' | 'profile' | 'article';
  canonical?: string;
  includeProfileTags?: boolean;
}

/**
 * Generate SEO meta tags for a page using React 19 native metadata
 * Usage: Place the returned JSX elements at the top of your component
 */
export function SEOMetaTags({
  path,
  title: propTitle,
  description: propDescription,
  keywords: propKeywords,
  ogType: propOgType,
  canonical,
  includeProfileTags: propIncludeProfileTags,
}: SEOMetaTagsProps) {
  const meta = routesMap[path] || routesMap['/'];
  const title = propTitle || meta.title;
  const description = propDescription || meta.description;
  const keywords = propKeywords || meta.keywords;
  const ogType = propOgType || (meta.ogType as 'website' | 'profile' | 'article') || 'website';
  const includeProfileTags = propIncludeProfileTags ?? meta.includeProfileTags ?? false;

  const siteUrl = typeof window !== 'undefined' ? window.location.origin : 'https://www.roberteklund.us';
  const fullTitle = `${title} | Robert Eklund`;
  const canonicalUrl = canonical || `${siteUrl}${path}`;

  return (
    <>
      {/* Basic SEO */}
      <title>{fullTitle}</title>
      <meta name="description" content={description} />
      {keywords && <meta name="keywords" content={keywords} />}
      <meta name="author" content="Robert Eklund" />
      <link rel="canonical" href={canonicalUrl} />

      {/* OpenGraph Tags */}
      <meta property="og:title" content={fullTitle} />
      <meta property="og:description" content={description} />
      <meta property="og:type" content={ogType} />
      <meta property="og:url" content={canonicalUrl} />
      <meta property="og:image" content={`${siteUrl}/headshot`} />
      <meta property="og:site_name" content="Robert Eklund Portfolio" />

      {/* OpenGraph Profile-specific tags */}
      {includeProfileTags && (
        <>
          <meta property="profile:first_name" content="Robert" />
          <meta property="profile:last_name" content="Eklund" />
        </>
      )}

      {/* Twitter Card Tags */}
      <meta name="twitter:card" content="summary" />
      <meta name="twitter:title" content={fullTitle} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={`${siteUrl}/headshot`} />
    </>
  );
}

/**
 * Generate JSON-LD Person schema for structured data
 * Only include on profile/about pages
 */
export function PersonSchema({ description: propDescription }: { description?: string }) {
  const siteUrl = typeof window !== 'undefined' ? window.location.origin : 'https://www.roberteklund.us';
  const meta = routesMap['/'];
  const description = propDescription || meta?.description || '';

  const personSchema = {
    '@context': 'https://schema.org',
    '@type': 'Person',
    name: 'Robert Eklund',
    jobTitle: 'Software Engineer',
    url: siteUrl,
    image: `${siteUrl}/headshot`,
    sameAs: [
      'https://github.com/Reklund3',
      'https://www.linkedin.com/in/robert-eklund-64302976/',
    ],
    knowsAbout: [
      'Rust',
      'Functional Programming',
      'TypeScript',
      'React',
      'Distributed Systems',
      'Actix Web',
      'PostgreSQL',
    ],
    alumniOf: {
      '@type': 'Organization',
      name: 'Texas State University',
    },
    description: description,
  };

  return (
    <script type="application/ld+json">
      {JSON.stringify(personSchema, null, 2)}
    </script>
  );
}
