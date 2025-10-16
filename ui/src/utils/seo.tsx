import React from 'react';

/**
 * SEO metadata utilities for React 19 native metadata support
 */

interface SEOMetaTagsProps {
  title: string;
  description: string;
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
  title,
  description,
  keywords,
  ogType = 'website',
  canonical,
  includeProfileTags = false,
}: SEOMetaTagsProps) {
  const siteUrl = typeof window !== 'undefined' ? window.location.origin : 'https://roberteklund.com';
  const fullTitle = `${title} | Robert Eklund`;
  const currentPath = typeof window !== 'undefined' ? window.location.pathname : '';
  const canonicalUrl = canonical || `${siteUrl}${currentPath}`;

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
export function PersonSchema({ description }: { description: string }) {
  const siteUrl = typeof window !== 'undefined' ? window.location.origin : 'https://roberteklund.com';

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
