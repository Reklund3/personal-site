import { Box, Button, CircularProgress, Collapse, Dialog, DialogActions, DialogContent, DialogTitle, Divider, FormControl, TextField, Typography } from "@mui/material";
import { CheckCircle, Dangerous } from "@mui/icons-material";
import DOMPurify from 'dompurify';
import React from "react";

interface ContactDialogProps {
    dialogOpen: boolean;
    onClose: () => void;
}

interface ContactRejected {
    reason: string;
}

const illegalNameChars =  ['/', '(', ')', '"', '<', '>', '\\', '{', '}']

function checkNameForIllegalChars(name: string) {
    const foundIllegalChars = [];
    for (const char of illegalNameChars) {
        if (name.includes(char)) {
            foundIllegalChars.push(char);
        }
    }
    return foundIllegalChars;
}

function debounce<T extends (...args: any[]) => any>(func: T, wait: number) {
    let timeout: ReturnType<typeof setTimeout> | null;

    return (...args: Parameters<T>) => {
        if (timeout) {
            clearTimeout(timeout)
        }

        timeout = setTimeout(() => {
            func(...args);
            timeout = null;
        }, wait);
    }
}

const validateName = (name: string) => {
    const invalidChars = checkNameForIllegalChars(name);
    if (name.length === 0) {
        return "This field is required";
    } else if (invalidChars.length > 0) {
        return `Name contains illegal characters: ${invalidChars.join(", ")}`;
    } else if (name.length > 256) {
        return `${name.length}/256 characters (${name.length - 256} too many)`;
    }
    return "";
};

const validateEmail = (email: string) => {
    if (email.length === 0) {
        return "This field is required";
    }
    // Add email format validation here if needed
    return "";
};

const validateMessage = (message: string) => {
    const sanitizedMessage = DOMPurify.sanitize(message);
    if (message.length === 0) {
        return "This field is required";
    } else if (sanitizedMessage.length === 0) {
        return "Invalid input. Please remove any HTML tags."
    } else if (sanitizedMessage.length != message.length) {
        return "Invalid input. Please remove any HTML tags."
    } else if (message.length > 1024) {  // Correct the character limit here
        return `${message.length}/1024 characters (${message.length - 1024} too many)`;
    }
    return "";
};

const ContactDialog: React.FC<ContactDialogProps> = ({dialogOpen, onClose}: ContactDialogProps) => {
    const [formData, setFormData] = React.useState({
        name: '',
        email: '',
        message: '',
    });

    const [touched, setTouched] = React.useState({
        name: false,
        email: false,
        message: false,
    });

    const [nameError, setNameError] = React.useState<string>('');
    const [emailError, setEmailError] = React.useState<string>('');
    const [messageError, setMessageError] = React.useState<string>('');

    const [isSubmitting, setIsSubmitting] = React.useState(false);
    const [requestStatus, setRequestStatus] = React.useState<{ success?: boolean, message?: string } | null>(null);

    React.useEffect(() => {
        if (touched.name) { // Only validate if the dialog is open
            setNameError(validateName(formData.name));
        }
    }, [formData.name, touched.name]);
    React.useEffect(() => {
        if (touched.email) { // Only validate if the dialog is open
            setEmailError(validateEmail(formData.email));
        }
    }, [formData.email, touched.email]);

    const debounceValidateMessage = React.useCallback(
        debounce(
            (message: string) => setMessageError(validateMessage(message)),
            500
        ),
        []
    );
    React.useEffect(() => {
        if (touched.message) { // Only validate if the dialog is open
            debounceValidateMessage(formData.message);
        }
    }, [formData.message, touched.message, debounceValidateMessage]);

    React.useEffect(() => {
        if (nameError === '' && emailError === '' && messageError === '' && formData.name.length > 0 && formData.email.length > 0 && formData.message.length > 0) {
            setIsFormValid(true);
        } else {
            setIsFormValid(false);
        }
    }, [nameError, emailError, messageError, formData.name, formData.email, formData.message]);

    const [isFormValid, setIsFormValid] = React.useState(false);

    const handleFormChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        setFormData({
            ...formData,
            [event.target.id]: event.target.value,
        });
        setTouched({ ...touched, [event.target.id]: true });
    }

    const handleClose = () => {
        setFormData({ name: '', email: '', message: '' });
        setNameError('');
        setEmailError('');
        setMessageError('');
        setIsFormValid(false);
        setIsSubmitting(false);
        setRequestStatus(null);
        setTouched({name: false, email: false, message: false});
        onClose();
    }

    const onSubmit: (formData: { name: string, email: string, message: string }) => Promise<void> = async data => {
        setIsSubmitting(true)
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 5000); // Timeout after 5 seconds

        // Small delay to smooth transitions for processing.
        await new Promise(resolve => setTimeout(resolve, 500)); // 1-second delay

        try {
            const response = await fetch('/contact', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(data),
                signal: controller.signal,
            });

            clearTimeout(timeoutId);

            if (response.ok) {
                setRequestStatus({ success: true, message: 'Thank you for reaching out. I look forward to speaking with you.' });
                //TODO: Update the dialog with a success
                console.log("Thank you for reaching out. I look forward to speaking with you.");
            } else {
                try {
                    const errorData: ContactRejected = await response.json();
                    setRequestStatus({ success: false, message: "Failed to submit contact information: " + errorData.reason });
                } catch (parseError) {
                    console.error("Failed to submit contact information. Error parsing response: ", parseError);
                    setRequestStatus({ success: false, message: "Failed to submit contact information. Please try again shortly." });
                }
            }
        } catch (error) {
            console.error("Failed to submit contact information. Error: ", error);
            if (error instanceof Error && error.name === 'AbortError') {
                setRequestStatus({ success: false, message: "Failed to submit contact information. Request timed out."});
            } else {
                setRequestStatus({ success: false, message: "Failed to submit contact information. Please try again later."});
            }
        } finally {
            clearTimeout(timeoutId);
            setIsSubmitting(false)
        }
    }

    const handleContactSubmit = () => {
        if (nameError === '' && emailError === '' && messageError === '') {
            onSubmit(formData);
        } else {
            console.log("Form submission failed. Errors: ", { nameError, emailError, messageError });
        }
    }

    return (
        <Dialog open={dialogOpen} onClose={handleClose}>
            <DialogTitle>Contact Form</DialogTitle>
            <Divider></Divider>
            {/* DialogContent */}
            <DialogContent>
                <Collapse in={isSubmitting}>
                    <Box
                        sx={{
                            display: "flex",
                            flexDirection: "column",
                            alignItems: "center",
                            justifyContent: "center",
                            height: "100%"
                        }}>
                        <CircularProgress size={40} />
                    </Box>
                </Collapse>

                <Collapse in={!!requestStatus}>
                    <Box
                        sx={{
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            height: "100%"
                        }}>
                        {requestStatus?.success ? (
                            <Box
                                sx={{
                                    display: "flex",
                                    flexDirection: "row",
                                    alignItems: "center",
                                    justifyContent: "center"
                                }}>
                                <CheckCircle sx={{ color: 'success.main', m:1 }} />
                                <Typography variant="body1" align="center">{requestStatus.message}</Typography>
                            </Box>
                        ) : (
                            <Box
                                sx={{
                                    display: "flex",
                                    flexDirection: "row",
                                    alignItems: "center",
                                    justifyContent: "center"
                                }}>
                                <Dangerous sx={{ color: 'error.main', m:1 }} />
                                <Typography variant="body1" align="center">{requestStatus?.message}</Typography>
                            </Box>
                        )}
                    </Box>
                </Collapse>

                <Collapse in={!requestStatus && !isSubmitting}>
                    <Box>
                        <FormControl fullWidth>
                            <TextField
                                autoFocus
                                margin="dense"
                                id="name"
                                label="Name"
                                type="text"
                                value={formData.name}
                                onChange={handleFormChange}
                                fullWidth
                                helperText={
                                    nameError || `${formData.name.length}/256 characters`
                                }
                                error={!!nameError}
                                sx = {{
                                    "& legend": {
                                        transition: "unset",
                                    }
                                }}
                            />
                        </FormControl>
                        <FormControl fullWidth>
                            <TextField
                                margin="dense"
                                id="email"
                                label="Email Address"
                                type="email"
                                value={formData.email}
                                onChange={handleFormChange}
                                fullWidth
                                helperText={emailError}
                                error={!!emailError}
                            />
                        </FormControl>
                        <FormControl fullWidth>
                            <TextField
                                margin="dense"
                                id="message"
                                label="Message"
                                type="text"
                                multiline
                                rows={3}
                                value={formData.message}
                                onChange={handleFormChange}
                                fullWidth
                                helperText={messageError || `${formData.message.length}/1024 characters`}
                                error={!!messageError}
                                sx = {{
                                    "& legend": {
                                        transition: "unset",
                                    }
                                }}
                            />
                        </FormControl>
                    </Box>
                </Collapse>
            </DialogContent>
            {/* DialogActions */}
            {!requestStatus && !isSubmitting &&
                <DialogActions>
                    <Button onClick={handleClose} disabled={isSubmitting} variant="contained" color="error">Cancel</Button>
                    <Button onClick={handleContactSubmit} disabled={isSubmitting || !isFormValid} variant="contained">Send</Button>
                </DialogActions>
            }
            {isSubmitting &&
                <DialogActions>
                    <Button onClick={handleClose} disabled={true} variant="contained">Close</Button>
                </DialogActions>
            }
            {requestStatus &&
                <DialogActions>
                    <Button
                        onClick={handleClose}
                        color={requestStatus ? (requestStatus.success ? "success" : "error") : "primary"}
                        variant="contained"
                    >
                        Close
                    </Button>
                </DialogActions>
            }
        </Dialog>
    );
}

export default ContactDialog;