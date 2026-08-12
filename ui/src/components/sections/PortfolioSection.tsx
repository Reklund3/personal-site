import React from 'react';
import Masonry from '@mui/lab/Masonry';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardActions from '@mui/material/CardActions';
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
        {allProjects.map((project, idx) => (
          <Card key={idx}>
            <CardContent
              sx={{
                pb: 0,
              }}
            >
              {/* Title as h3 */}
              <Typography
                component="h3"
                sx={{
                  fontSize: 13.5,
                  fontWeight: 'bold',
                  color: 'rgba(255,255,255,.9)',
                  mb: project.subheader ? '2px' : '8px',
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
                    mb: '8px',
                  }}
                >
                  {paragraph}
                </Typography>
              ))}
            </CardContent>

            {/* Link in CardActions */}
            <CardActions
              sx={{
                p: 0,
                pt: '8px',
              }}
            >
              <Link
                href={project.link}
                color="primary"
                underline="none"
                target="_blank"
                rel="noopener"
                sx={{
                  fontSize: 12,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  '&:hover': {
                    textDecoration: 'underline',
                  },
                }}
              >
                {project.linkLabel ?? 'View on GitHub'}
                <span>→</span>
              </Link>
            </CardActions>
          </Card>
        ))}
      </Masonry>
    </Section>
  );
}
