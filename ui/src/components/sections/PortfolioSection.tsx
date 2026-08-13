import React from 'react';
import Masonry from '@mui/lab/Masonry';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Link from '@mui/material/Link';
import Typography from '@mui/material/Typography';
import Section from './Section';
import { CONTENT } from '../../content';

export default function PortfolioSection() {
  // Combine personal and open-source projects in order
  const allProjects = [...CONTENT.portfolio.personal, ...CONTENT.portfolio.openSource];

  return (
    <Section id="portfolio" eyebrow="PORTFOLIO" band="default" eyebrowGap={18}>
      <Masonry
        columns={{ xs: 1, md: 2 }}
        spacing={1.75}
        sequential
        defaultColumns={2}
        defaultSpacing={1.75}
      >
        {allProjects.map((project) => (
          <Card key={project.title}>
            {/*
              The handoff card is a SINGLE container with a flat 16px padding
              (markup line 484): title, paragraphs and the link are all siblings at
              the same inset. There is no actions region — an earlier version used
              <CardActions>, which introduced its own box and left the link flush
              against the card border. MUI's default `&:last-child` bumps the bottom
              padding to 24px, so it is pinned back to 16px here.
            */}
            <CardContent
              sx={{
                p: '16px',
                '&:last-child': { pb: '16px' },
              }}
            >
              {/* Title as h3 */}
              <Typography
                component="h3"
                sx={{
                  fontSize: 13.5,
                  fontWeight: 'bold',
                  color: 'rgba(255,255,255,.9)',
                  // Handoff: the title carries mb 8px on its own, but drops to 0 when
                  // a subheader follows, because the subheader then owns the 8px.
                  mb: project.subheader ? 0 : '8px',
                }}
              >
                {project.title}
              </Typography>

              {/* Optional subheader */}
              {project.subheader && (
                <Typography
                  sx={{
                    fontSize: 11,
                    color: 'rgba(255,255,255,.5)',
                    mb: '8px',
                  }}
                >
                  {project.subheader}
                </Typography>
              )}

              {/* Paragraphs */}
              {project.paragraphs.map((paragraph, pidx) => (
                <Typography
                  key={pidx}
                  sx={{
                    fontSize: 12,
                    lineHeight: 1.6,
                    color: 'rgba(255,255,255,.7)',
                    mb: '6px',
                  }}
                >
                  {paragraph}
                </Typography>
              ))}

              {/*
                Inline <a>, the last child of the same padded box — not a flex row in
                a separate actions region. 11px per the handoff; the arrow is part of
                the link text, so it never wraps onto its own line.
              */}
              <Link
                href={project.link}
                color="primary"
                underline="none"
                target="_blank"
                // noreferrer as well as noopener, matching the footer's external
                // links — suppresses the Referer header to the destination.
                rel="noopener noreferrer"
                // Two cards fall back to the same "View on GitHub" text, so a
                // screen reader listing links hears one name twice with no way to
                // tell them apart. The visible text stays short; the accessible
                // name carries the project.
                aria-label={`${project.linkLabel ?? 'View on GitHub'}: ${project.title}`}
                sx={{
                  fontSize: 11,
                  '&:hover': {
                    textDecoration: 'underline',
                  },
                }}
              >
                {project.linkLabel ?? 'View on GitHub'} &#8594;
              </Link>
            </CardContent>
          </Card>
        ))}
      </Masonry>
    </Section>
  );
}
