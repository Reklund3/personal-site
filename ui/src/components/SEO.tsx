import React from 'react';
import { Helmet } from 'react-helmet-async';

interface SEOProps {
  title: string;
  description: string;
  keywords?: string;
  ogType?: 'website' | 'profile' | 'article';
  canonical?: string;
  includePersonSchema?: boolean;
}

export const SEO: React.FC<SEOProps> = ({
  title,
  description,
  keywords,
  ogType = 'website',
  canonical,
  includePersonSchema = false,
}) => {
  const siteUrl = window.location.origin;
  const fullTitle = `${title} | Robert Eklund`;
  const canonicalUrl = canonical || `${siteUrl}${window.location.pathname}`;

  // Person Schema (JSON-LD) for structured data
  const personSchema = includePersonSchema
    ? {
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
          name: 'Stony Brook University',
        },
        description: description,
      }
    : null;

  return (
    <Helmet>
      {/* Basic SEO - Task 12 */}
      <title>{fullTitle}</title>
      <meta name="description" content={description} />
      {keywords && <meta name="keywords" content={keywords} />}
      <meta name="author" content="Robert Eklund" />
      <link rel="canonical" href={canonicalUrl} />

      {/* OpenGraph Tags - Task 14 */}
      <meta property="og:title" content={fullTitle} />
      <meta property="og:description" content={description} />
      <meta property="og:type" content={ogType} />
      <meta property="og:url" content={canonicalUrl} />
      <meta property="og:image" content={`${siteUrl}/headshot`} />
      <meta property="og:site_name" content="Robert Eklund Portfolio" />

      {/* OpenGraph Profile-specific tags */}
      {ogType === 'profile' && (
        <>
          <meta property="profile:first_name" content="Robert" />
          <meta property="profile:last_name" content="Eklund" />
        </>
      )}

      {/* Twitter Card Tags - Task 14 */}
      <meta name="twitter:card" content="summary" />
      <meta name="twitter:title" content={fullTitle} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={`${siteUrl}/headshot`} />

      {/* Structured Data (JSON-LD) - Task 13 */}
      {personSchema && (
        <script type="application/ld+json">
          {JSON.stringify(personSchema, null, 2)}
        </script>
      )}
    </Helmet>
  );
};
