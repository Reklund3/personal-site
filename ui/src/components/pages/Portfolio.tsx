import React from 'react';
import { Box, Card, CardActions, CardContent, CardHeader, Divider, IconButton, Link, Paper, Typography } from '@mui/material';
import GitHubIcon from "@mui/icons-material/GitHub";
import CodeIcon from "@mui/icons-material/Code";
import { SEOMetaTags } from '../../utils/seo';

export default function Portfolio() {
    return (
        <>
            <SEOMetaTags
                title="Portfolio"
                description="Personal projects and open source contributions. Built with Rust, TypeScript, React, and Scala. Includes contributions to Akka ActorTestkit and personal web applications."
                keywords="Portfolio, Personal Projects, Open Source, Rust Projects, React Projects, Akka, Scala, TypeScript, Web Development"
            />
            <Box sx={{ my: 4 }}>
            <Paper elevation={5} square={false} sx={{ p: 3 }}>
                {/* Personal Projects Section */}
                <Typography variant="h5" component="h2" sx={{ mb: 3, fontWeight: 600 }}>
                    Personal Projects
                </Typography>
                <Box sx={{ display: 'grid', gap: 2, gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', mb: 5 }}>
                    <Card sx={{ display: 'flex', flexDirection: 'column' }}>
                        <CardHeader title="Zero2Prod" />
                        <Divider variant="fullWidth" />
                        <CardContent>
                            This is the website you are currently on. It is a React/Typescript/Material UI application.
                        </CardContent>
                        <CardContent>
                            It is a simple portfolio website that I created to showcase my skills and projects.
                        </CardContent>
                        <Box sx={{ flexGrow: 1 }}/>
                        <CardActions>
                            <IconButton href="https://github.com/Reklund3/personal-site" target="_blank" rel="noopener noreferrer">
                                <GitHubIcon />
                            </IconButton>
                        </CardActions>
                    </Card>

                    <Card sx={{ display: 'flex', flexDirection: 'column' }}>
                        <CardHeader title="Posts" />
                        <Divider variant="fullWidth" />
                        <CardContent>
                            Akka post service to manage posts and comments. This project was started as a learning
                            exercise to learn more about Akka and Akka HTTP. It evolved into learning about gRPC.
                        </CardContent>
                        <CardContent>
                            I intend to circle back to this project and convert it to Pekko.
                        </CardContent>
                        <Box sx={{ flexGrow: 1 }}/>
                        <CardActions>
                            <IconButton href="https://gitlab.com/Reklund3/posts" target="_blank" rel="noopener noreferrer">
                                <GitHubIcon />
                            </IconButton>
                        </CardActions>
                    </Card>

                    <Card sx={{ display: 'flex', flexDirection: 'column' }}>
                        <CardHeader title="Posts-App" />
                        <Divider variant="fullWidth" />
                        <CardContent>
                            A project I create to explore the Tauri framework. This is a simple that uses the
                            Posts service and use various crate to integrate with gRPC transport to the Posts service.
                        </CardContent>
                        <CardContent>
                            I don't currently have any additional plans for this project but, it has been a great
                            learning experience.
                        </CardContent>
                        <Box sx={{ flexGrow: 1 }}/>
                        <CardActions>
                            <IconButton href="https://github.com/Reklund3/posts-app/tree/init" target="_blank" rel="noopener noreferrer">
                                <GitHubIcon />
                            </IconButton>
                        </CardActions>
                    </Card>
                </Box>

                <Divider sx={{ my: 4 }} />

                {/* Open Source Contributions Section */}
                <Typography variant="h5" component="h2" sx={{ mb: 3, fontWeight: 600 }}>
                    Open Source Contributions
                </Typography>
                <Box sx={{ display: 'grid', gap: 2, gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))' }}>
                    <Card sx={{ display: 'flex', flexDirection: 'column' }}>
                        <CardHeader
                            title="Akka ActorTestkit"
                            subheader="Factory Methods Enhancement"
                        />
                        <Divider variant="fullWidth" />
                        <CardContent>
                            Contributed factory methods to the Akka ActorTestkit framework, improving distributed
                            system test workflows.
                        </CardContent>
                        <CardContent>
                            Enhanced functionality for Typed Actor TestKit, making it easier for developers to
                            write comprehensive tests for actor-based systems.
                        </CardContent>
                        <Box sx={{ flexGrow: 1 }}/>
                        <CardActions>
                            <IconButton
                                href="https://github.com/akka/akka/pull/28871"
                                target="_blank"
                                rel="noopener noreferrer"
                                aria-label="View Pull Request"
                            >
                                <CodeIcon />
                            </IconButton>
                            <Link
                                href="https://github.com/akka/akka/pull/28871"
                                target="_blank"
                                rel="noopener noreferrer"
                                sx={{ ml: 1, fontSize: '0.875rem' }}
                            >
                                View PR #28871
                            </Link>
                        </CardActions>
                    </Card>
                </Box>
            </Paper>
        </Box>
        </>
    );
}
