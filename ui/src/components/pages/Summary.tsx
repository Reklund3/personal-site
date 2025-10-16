import React from 'react';
import { Box, Paper, Typography, Divider } from '@mui/material';
import { SEOMetaTags, PersonSchema } from '../../utils/seo';

export default function Summary() {
    const description = "Software engineer with expertise in Rust, functional programming, and distributed systems. Former finance professional turned full-stack developer passionate about type safety and scalable infrastructure.";

    return (
        <>
            <SEOMetaTags
                title="About Me"
                description={description}
                keywords="Robert Eklund, Software Engineer, Rust, Functional Programming, TypeScript, React, Actix Web, Scala, DevOps, Backend Developer"
                ogType="profile"
                includeProfileTags={true}
            />
            <PersonSchema description={description} />
            <Box component="main" sx={{ my: 4 }}>
            <Paper component="article" elevation={5} square={false} sx={{ p: 3 }}>
                {/* Professional Summary */}
                <Typography variant="h5" component="h1" sx={{ mb: 3, fontWeight: 600 }}>
                    About Me
                </Typography>
                <Typography variant="body1" sx={{ mb: 4, fontSize: '1.1rem', lineHeight: 1.8 }}>
                    Software engineer with a unique background spanning finance, DevOps, and full-stack development.
                    Passionate about type systems, functional programming, and building scalable infrastructure.
                    Driven by performance optimization, mentorship, and fostering inclusive engineering cultures.
                </Typography>

                <Divider sx={{ my: 4 }} />

                <Box component="section" aria-labelledby="background-heading">
                    {/* Background */}
                    <Typography variant="h6" component="h2" id="background-heading" sx={{ mb: 2, fontWeight: 600 }}>
                        Background
                    </Typography>
                    <Typography variant="body1" sx={{ mb: 2, lineHeight: 1.7 }}>
                        Leaving a secure finance career to pursue software engineering felt like jumping off a cliff,
                        but it was a jump I had to make. My background in finance instilled analytical and critical
                        thinking skills that proved invaluable when refactoring codebases and reducing technical debt.
                        After completing Austin Coding Academy, I landed a role at Cloud Imperium Games as an Associate
                        DevOps Engineer, diving headfirst into Scala, event-driven architecture, and distributed systems.
                    </Typography>
                </Box>

                <Divider sx={{ my: 4 }} />

                <Box component="section" aria-labelledby="technical-journey-heading">
                    {/* Technical Journey */}
                    <Typography variant="h6" component="h2" id="technical-journey-heading" sx={{ mb: 2, fontWeight: 600 }}>
                        Technical Journey
                    </Typography>
                <Typography variant="body1" sx={{ mb: 2, lineHeight: 1.7 }}>
                    Coming from C# and JavaScript, Scala opened up a new world of functional programming concepts:
                </Typography>
                <Box component="ul" sx={{ mb: 2, pl: 3, '& li': { mb: 1 } }}>
                    <li>
                        <Typography variant="body2">
                            <strong>Functional Programming:</strong> Monads, pure functions, higher-order functions,
                            and tail recursion optimization
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            <strong>Type Systems:</strong> Leveraging types to express guarantees and make invalid
                            states unrepresentable
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            <strong>Advanced Libraries:</strong> Refined for compile-time validation, Cats for
                            functional abstractions, NonEmpty collections
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            <strong>Language Evolution:</strong> Transitioned from Scala 2 implicits to Scala 3 givens
                        </Typography>
                    </li>
                </Box>
                <Typography variant="body1" sx={{ mb: 2, lineHeight: 1.7 }}>
                    These concepts fundamentally changed how I approach software design, leading me to mentor team
                    members on type-driven development and proper test coverage beyond golden path scenarios.
                </Typography>
                </Box>

                <Divider sx={{ my: 4 }} />

                {/* Key Achievements */}
                <Typography variant="h6" component="h3" sx={{ mb: 2, fontWeight: 600 }}>
                    Key Achievements
                </Typography>
                <Box component="ul" sx={{ mb: 2, pl: 3, '& li': { mb: 1.5 } }}>
                    <li>
                        <Typography variant="body2">
                            Modernized legacy systems and migrated a monolithic internal webservice to microservices
                            architecture
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            Contributed to both ScalaJS/Play frontend and Lagom/Akka backend implementations
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            Developed critical internal Scala libraries used across multiple team services
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            Optimized application performance through benchmarking and Docker container size reduction
                        </Typography>
                    </li>
                </Box>

                <Divider sx={{ my: 4 }} />

                {/* DevOps & Infrastructure */}
                <Typography variant="h6" component="h3" sx={{ mb: 2, fontWeight: 600 }}>
                    DevOps & Infrastructure
                </Typography>
                <Typography variant="body1" sx={{ mb: 2, lineHeight: 1.7 }}>
                    Led development of microservices CI/CD pipelines and infrastructure modernization:
                </Typography>
                <Box component="ul" sx={{ mb: 2, pl: 3, '& li': { mb: 1 } }}>
                    <li>
                        <Typography variant="body2">
                            Deployed and configured GitLab runners for automated builds and deployments
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            Managed GitLab container registry for Docker images and package registry for Helm charts
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            Orchestrated EKS deployments, upgrading from Helm 2 to Helm 3
                        </Typography>
                    </li>
                    <li>
                        <Typography variant="body2">
                            Implemented GitOps workflows with ArgoCD for automated deployments and rollback capabilities
                        </Typography>
                    </li>
                </Box>

                <Divider sx={{ my: 4 }} />

                {/* Mentorship & Collaboration */}
                <Typography variant="h6" component="h3" sx={{ mb: 2, fontWeight: 600 }}>
                    Mentorship & Collaboration
                </Typography>
                <Typography variant="body1" sx={{ mb: 2, lineHeight: 1.7 }}>
                    Beyond technical work, I'm passionate about helping others grow and fostering collaborative
                    environments. My favorite aspect of software engineering is working with talented, passionate
                    people who share knowledge freely.
                </Typography>
                <Typography variant="body1" sx={{ mb: 2, lineHeight: 1.7 }}>
                    I believe explaining concepts to others refines our own understanding and strengthens the team.
                    More importantly, we should actively work to foster cultures of inclusion, continuous learning,
                    and mutual growth.
                </Typography>

                <Divider sx={{ my: 4 }} />

                {/* Future Focus */}
                <Typography variant="h6" component="h3" sx={{ mb: 2, fontWeight: 600 }}>
                    Future Focus
                </Typography>
                <Typography variant="body1" sx={{ mb: 2, lineHeight: 1.7 }}>
                    The rise of LLMs and AI tools has captured my attention. Tools like Cursor, Windsurf, and
                    JetBrains' AI assistants are revolutionizing how we build software, enabling faster prototyping,
                    better context search, and exploration of alternative solutions.
                </Typography>
                <Typography variant="body1" sx={{ lineHeight: 1.7 }}>
                    I look forward to the challenges ahead and opportunities to collaborate with like-minded
                    technologists. If you're interested in connecting, reach out via the <strong>Contact Now</strong> button
                    or find me on LinkedIn.
                </Typography>
            </Paper>
        </Box>
        </>
    );
}
