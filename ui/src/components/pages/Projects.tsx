import React from 'react';
import { Box, Card, CardActions, CardContent, CardHeader, Divider, IconButton, Paper } from '@mui/material';
import GitHubIcon from "@mui/icons-material/GitHub";

export default function Projects() {
    return (
        <Box sx={{ my: 4 }}>
            <Paper elevation={5} square={false} sx={{ p: 2 }}>
                <Box sx={{ display: 'grid', gap: 0, gridTemplateColumns: 'repeat(auto-fit, minmax(33%, 1fr))' }}>
                    <Card sx={{ m: 2, display: 'flex', flexDirection: 'column' }}>
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
                            <IconButton sx={{ display: { xs: 'flex', md: 'flex' } }} href={"https://github.com/Reklund3/personal-site"}>
                                <GitHubIcon sx={{ display: { xs: 'flex', md: 'flex' } }} />
                            </IconButton>
                        </CardActions>
                    </Card>
                    <Card sx={{ m: 2, display: 'flex', flexDirection: 'column' }}>
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
                            <IconButton sx={{ display: { xs: 'flex', md: 'flex' } }} href={"https://gitlab.com/Reklund3/posts"}>
                                <GitHubIcon sx={{ display: { xs: 'flex', md: 'flex' } }} />
                            </IconButton>
                        </CardActions>
                    </Card>
                    <Card sx={{ m: 2, display: 'flex', flexDirection: 'column' }}>
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
                            <IconButton sx={{ display: { xs: 'flex', md: 'flex' } }} href={"https://github.com/Reklund3/posts-app/tree/init"}>
                                <GitHubIcon sx={{ display: { xs: 'flex', md: 'flex' } }} />
                            </IconButton>
                        </CardActions>
                    </Card>
                </Box>
            </Paper>
        </Box>
    );
}