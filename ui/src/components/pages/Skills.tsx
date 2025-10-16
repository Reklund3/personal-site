import React from 'react';
import { Box, Paper, Typography, Chip, Stack } from '@mui/material';
import CodeIcon from '@mui/icons-material/Code';
import CloudIcon from '@mui/icons-material/Cloud';
import StorageIcon from '@mui/icons-material/Storage';
import BuildIcon from '@mui/icons-material/Build';
import ConstructionIcon from '@mui/icons-material/Construction';
import { SEOMetaTags } from '../../utils/seo';

export default function Skills() {
    const programmingLanguages = [
        { name: 'Scala', color: '#DC322F' },
        { name: 'Rust', color: '#CE422B' },
        { name: 'Java', color: '#007396' },
        { name: 'C#', color: '#239120' },
        { name: 'C++', color: '#00599C' },
        { name: 'Go', color: '#00ADD8' },
    ];

    const cloudInfra = [
        { name: 'Docker', color: '#2496ED' },
        { name: 'Kubernetes', color: '#326CE5' },
        { name: 'Helm', color: '#0F1689' },
        { name: 'Argo', color: '#EF7B4D' },
        { name: 'AWS EKS', color: '#FF9900' },
        { name: 'GKE', color: '#4285F4' },
    ];

    const frameworks = [
        { name: 'Pekko', color: '#1976d2' },
        { name: 'Akka', color: '#1976d2' },
        { name: 'Play', color: '#92D13D' },
        { name: 'Okta', color: '#007DC1' },
        { name: 'ScalaJS', color: '#DC322F' },
        { name: 'Diode', color: '#1976d2' },
        { name: 'Git', color: '#F05032' },
        { name: 'GitLab', color: '#FC6D26' },
        { name: 'GitHub', color: '#181717' },
    ];

    const databases = [
        { name: 'PostgreSQL', color: '#336791' },
        { name: 'MySQL', color: '#4479A1' },
        { name: 'Cassandra', color: '#1287B1' },
        { name: 'Kafka', color: '#231F20' },
    ];

    const devops = [
        { name: 'GitHub Actions', color: '#2088FF' },
        { name: 'GitLab CI', color: '#FC6D26' },
        { name: 'Helm Charts', color: '#0F1689' },
        { name: 'Docker Pipelines', color: '#2496ED' },
    ];

    const otherTools = [
        { name: 'JetBrains Suite', color: '#000000' },
        { name: 'AWS', color: '#FF9900' },
        { name: 'Miro', color: '#FFD02F' },
        { name: 'Jira', color: '#0052CC' },
        { name: 'Atlassian', color: '#0052CC' },
    ];

    return (
        <>
            <SEOMetaTags
                title="Skills"
                description="Technical skills including Rust, TypeScript, React, Scala, PostgreSQL, and DevOps tools. Experienced in functional programming, distributed systems, and cloud infrastructure."
                keywords="Rust, TypeScript, React, Scala, PostgreSQL, DevOps, Kubernetes, Docker, GitLab CI, AWS, Functional Programming"
            />
            <Box sx={{ my: 4 }}>
                <Paper elevation={5} square={false} sx={{ p: 3 }}>
                    <Typography variant="h5" component="h5" sx={{mb: 3, textDecoration: 'underline'}}>
                        Soft Skills
                    </Typography>
                    <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{mb: 4}}>
                        <Chip label="Team Player" variant="outlined" />
                        <Chip label="Problem Solver" variant="outlined" />
                        <Chip label="Strategic Thinking" variant="outlined" />
                        <Chip label="Communication" variant="outlined" />
                        <Chip label="Leadership" variant="outlined" />
                    </Stack>

                    <Typography variant="h5" component="h5" sx={{mb: 3, textDecoration: 'underline'}}>
                        Technical Skills
                    </Typography>

                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                        <CodeIcon sx={{ mr: 1, color: 'text.secondary' }} />
                        <Typography variant="h6">Programming Languages</Typography>
                    </Box>
                    <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{mb: 3}}>
                        {programmingLanguages.map(lang => (
                            <Chip
                                key={lang.name}
                                label={lang.name}
                                sx={{
                                    bgcolor: lang.color,
                                    color: 'white',
                                    fontWeight: 500,
                                    '&:hover': {
                                        bgcolor: lang.color,
                                        opacity: 0.85,
                                    }
                                }}
                            />
                        ))}
                    </Stack>

                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                        <CloudIcon sx={{ mr: 1, color: 'text.secondary' }} />
                        <Typography variant="h6">Cloud & Infrastructure</Typography>
                    </Box>
                    <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{mb: 3}}>
                        {cloudInfra.map(tech => (
                            <Chip
                                key={tech.name}
                                label={tech.name}
                                sx={{
                                    bgcolor: tech.color,
                                    color: 'white',
                                    fontWeight: 500,
                                    '&:hover': {
                                        bgcolor: tech.color,
                                        opacity: 0.85,
                                    }
                                }}
                            />
                        ))}
                    </Stack>

                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                        <BuildIcon sx={{ mr: 1, color: 'text.secondary' }} />
                        <Typography variant="h6">Frameworks & Tools</Typography>
                    </Box>
                    <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{mb: 3}}>
                        {frameworks.map(tech => (
                            <Chip
                                key={tech.name}
                                label={tech.name}
                                sx={{
                                    bgcolor: tech.color,
                                    color: 'white',
                                    fontWeight: 500,
                                    '&:hover': {
                                        bgcolor: tech.color,
                                        opacity: 0.85,
                                    }
                                }}
                            />
                        ))}
                    </Stack>

                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                        <StorageIcon sx={{ mr: 1, color: 'text.secondary' }} />
                        <Typography variant="h6">Databases</Typography>
                    </Box>
                    <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{mb: 3}}>
                        {databases.map(tech => (
                            <Chip
                                key={tech.name}
                                label={tech.name}
                                sx={{
                                    bgcolor: tech.color,
                                    color: 'white',
                                    fontWeight: 500,
                                    '&:hover': {
                                        bgcolor: tech.color,
                                        opacity: 0.85,
                                    }
                                }}
                            />
                        ))}
                    </Stack>

                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                        <ConstructionIcon sx={{ mr: 1, color: 'text.secondary' }} />
                        <Typography variant="h6">DevOps & CI/CD</Typography>
                    </Box>
                    <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{mb: 3}}>
                        {devops.map(tech => (
                            <Chip
                                key={tech.name}
                                label={tech.name}
                                sx={{
                                    bgcolor: tech.color,
                                    color: 'white',
                                    fontWeight: 500,
                                    '&:hover': {
                                        bgcolor: tech.color,
                                        opacity: 0.85,
                                    }
                                }}
                            />
                        ))}
                    </Stack>

                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                        <BuildIcon sx={{ mr: 1, color: 'text.secondary' }} />
                        <Typography variant="h6">Other Tools</Typography>
                    </Box>
                    <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                        {otherTools.map(tech => (
                            <Chip
                                key={tech.name}
                                label={tech.name}
                                sx={{
                                    bgcolor: tech.color,
                                    color: 'white',
                                    fontWeight: 500,
                                    '&:hover': {
                                        bgcolor: tech.color,
                                        opacity: 0.85,
                                    }
                                }}
                            />
                        ))}
                    </Stack>
                </Paper>
            </Box>
        </>
    );
}