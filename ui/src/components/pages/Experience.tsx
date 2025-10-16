import React from 'react';
import { Box, Card, CardContent, CardHeader, Divider, Paper, Typography } from '@mui/material';
import { SEOMetaTags } from '../../utils/seo';

export default function Experience() {
    return (
        <>
            <SEOMetaTags
                title="Experience"
                description="Senior Micro-Service Engineer at Cloud Imperium Games. Experience in Scala, event-driven architecture, DevOps, and full-stack development. Background includes Agile IT and finance."
                keywords="Cloud Imperium Games, Senior Engineer, Scala, Microservices, DevOps, Event-Driven Architecture, Full-Stack Development"
            />
            <Box sx={{ my: 4 }}>
            <Paper elevation={5} square={false} sx={{ p: 2 }}>
                <Card sx={{ m: 2 }}>
                    <CardHeader title="Senior Micro-Service Engineer" subheader="Cloud Imperium Games - 03/2022 - Present"/>
                    <Divider variant="fullWidth" />
                    <CardContent>
                        <Typography sx={{ p: 1 }}>
                            Led code reviews for full team across all domains, infrastructure, platform code, and frontend.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Held cross-functional meetings with stakeholders to gather technical requirements, reducing ambiguity and ensuring delivery on time. Additionally, managed feature request and bug reports.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Implemented Authorization (AuthZ) and Authentication (AuthN) for service offerings using Attribute-Based Access Control (ABAC), securing systems against unauthorized access.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Developed and maintained internal Scala libraries depended on by a large portion of the teams internal services.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Investigated FFI implementations with Bindgen to work on a port of Pekko to Rust. This would allow for keeping the current Scala tests and strangle thorn Scala/java into Rust.
                        </Typography>
                    </CardContent>
                </Card>
                <Card sx={{ m: 2 }}>
                    <CardHeader title="Micro-Service Engineer" subheader="Cloud Imperium Games - 03/2021 - 03/2022" />
                    <Divider variant="fullWidth" />
                    <CardContent>
                        <Typography sx={{ p: 1 }}>
                            Proactively mentored junior engineers in system design, containerization, and deployment best practices.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Designed and deployed Kubernetes-based micro-services, increasing scalability and fault tolerance.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Enhanced deployment automation using Helm, reducing deployment times while maintaining system reliability.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Contributed and maintained full-stack efforts on the teams internal offerings.
                        </Typography>
                    </CardContent>
                </Card>
                <Card sx={{ m: 2 }}>
                    <CardHeader title="DevOps Engineer" subheader="Cloud Imperium Games - 03/2020 - 03/2021" />
                    <Divider variant="fullWidth" />
                    <CardContent>
                        <Typography sx={{ p: 1 }}>
                            Contributed to the development and maintenance internal full stack software offering.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography  sx={{ p: 1 }}>
                            Developed CI/CD pipelines in GitLab to automate deployments, reducing failures and build times.
                        </Typography>
                        <Divider variant="fullWidth" />
                        <Typography sx={{ p: 1 }}>
                            Built and maintained scalable internal Docker images to improve productivity for integrated testing.
                        </Typography>
                    </CardContent>
                </Card>
                <Card sx={{ m: 2 }}>
                    <CardHeader title="Associate DevOps Engineer" subheader="Cloud Imperium Games - 03/2019 - 03/2020" />
                    <Divider variant="fullWidth" />
                    <CardContent>
                        coming soon
                    </CardContent>
                </Card>
            </Paper>
        </Box>
        </>
    );
}