import { Box, Button, CircularProgress, Collapse, Dialog, DialogActions, DialogContent, DialogTitle, Divider, FormControl, TextField, Typography } from "@mui/material";
import { CheckCircle, Dangerous } from "@mui/icons-material";
import React from "react";
import DOMPurify from 'dompurify';

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

function debounce<T extends (...args: never[]) => void>(func: T, wait: number) {
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

export const validateName = (name: string) => {
    const invalidChars = checkNameForIllegalChars(name);
    if (name.trim().length === 0) {
        return "This field is required";
    } else if (invalidChars.length > 0) {
        return `Name contains illegal characters: ${invalidChars.join(", ")}`;
    } else if (name.length > 256) {
        return `${name.length}/256 characters (${name.length - 256} too many)`;
    }
    return "";
};

/**
 * Reduce a string to plain text: strip every tag, then decode the entities the
 * sanitizer introduces so the result is comparable to — and displayable as — the
 * original characters.
 *
 * The decode step is what the previous implementation was missing. `sanitize()`
 * returns innerHTML, so it ESCAPES as well as strips: "R&D" comes back as
 * "R&amp;D". Without decoding, that is both longer than the input (breaking any
 * comparison against it) and wrong to render (the user would see a literal
 * "R&amp;D"). Re-parsing through a detached textarea is safe here precisely
 * because the sanitize call above has already removed every tag.
 */
export const sanitizeToText = (value: string) => {
    const stripped = DOMPurify.sanitize(value, { ALLOWED_TAGS: [], ALLOWED_ATTR: [] });
    const decoder = document.createElement('textarea');
    decoder.innerHTML = stripped;
    return decoder.value;
};

/**
 * The tag names present in `value`, so the error can name them the way
 * validateName names illegal characters. Parsed in a detached document — this is
 * never inserted anywhere, and parseFromString does not execute scripts.
 */
export function findHtmlTags(value: string) {
    const doc = new DOMParser().parseFromString(value, 'text/html');
    const tags = new Set<string>();
    doc.body.querySelectorAll('*').forEach((el) => tags.add(el.tagName.toLowerCase()));
    return [...tags];
}

export const validateEmail = (email: string) => {
    if (email.length === 0) {
        return "This field is required";
    }
    // Deliberately permissive — just enough shape to catch a typo before a
    // round-trip. The server is the authority on whether an address is valid.
    // Reported in the same spirit as validateName: say which part is wrong.
    const invalidChars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', ',', ';']
        .filter((c) => email.includes(c));
    if (invalidChars.length > 0) {
        return `Email contains illegal characters: ${invalidChars.join(", ")}`;
    }
    if (!email.includes("@")) {
        return "Email address is missing an @";
    }
    if (email.split("@").length > 2) {
        return "Email address has more than one @";
    }
    const [local, domain] = email.split("@");
    if (local.length === 0) {
        return "Email address is missing the part before the @";
    }
    if (domain.length === 0) {
        return "Email address is missing the domain after the @";
    }
    if (!domain.includes(".")) {
        return "Email domain is missing a dot, e.g. example.com";
    }
    if (/\s/.test(email)) {
        return "Email address cannot contain spaces";
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
        return "Please enter a valid email address, e.g. name@example.com";
    }
    return "";
};

export const validateMessage = (message: string) => {
    // DOMPurify is the authority on what would be stripped; DOMParser only supplies
    // tag names so the user is told exactly what to remove rather than left to
    // guess — the same contract validateName keeps for illegal characters.
    //
    // The two markup branches below are the original pair of checks: "nothing
    // survives sanitizing" and "something was stripped". They are kept apart
    // because they are different user mistakes and deserve different advice.
    //
    // What changed is only the comparison. Testing `sanitize(message).length !==
    // message.length` fires on any message containing & < or >, because sanitize()
    // returns innerHTML and escapes as well as strips — "R&D" comes back as
    // "R&amp;D", longer than it went in. Comparing decoded text to the original
    // tests what was actually meant.
    const asText = sanitizeToText(message);

    if (message.length === 0) {
        return "This field is required";
    } else if (message.trim().length === 0) {
        return "This field is required";
    } else if (asText.trim().length === 0) {
        // Nothing at all survives — the message is markup end to end, so naming
        // individual tags is not the useful advice here.
        return "Your message is entirely HTML and would arrive empty. Please write it as plain text.";
    } else if (asText !== message) {
        const tags = findHtmlTags(message);
        if (tags.length > 0) {
            return `Message contains HTML tags: ${tags.map((t) => `<${t}>`).join(", ")}. Please remove them.`;
        }
        // Stripped something DOMParser does not surface as an element — a comment,
        // or a dangerous attribute on nothing nameable.
        return "Invalid input. Please remove any HTML markup.";
    } else if (message.length > 1024) {
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

    const [messageError, setMessageError] = React.useState<string>('');

    const [isSubmitting, setIsSubmitting] = React.useState(false);
    const [requestStatus, setRequestStatus] = React.useState<{ success?: boolean, message?: string } | null>(null);

    // Derived errors
    const nameError = touched.name ? validateName(formData.name) : '';
    const emailError = touched.email ? validateEmail(formData.email) : '';

    const debounceValidateMessage = React.useMemo(
        () => debounce(
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

    // Derived form validity
    const isFormValid = nameError === '' && emailError === '' && messageError === '' && formData.name.length > 0 && formData.email.length > 0 && formData.message.length > 0;

    const handleFormChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        setFormData({
            ...formData,
            [event.target.id]: event.target.value,
        });
        setTouched({ ...touched, [event.target.id]: true });
    }

    const handleClose = () => {
        setFormData({ name: '', email: '', message: '' });
        setMessageError('');
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
                    // The one server-controlled string that reaches the DOM. React
                    // escapes text children, so this is not an XSS sink today —
                    // but a poisoned record reaching the error path should not be
                    // able to smuggle markup into the dialog if that ever changes.
                    setRequestStatus({
                        success: false,
                        message: "Failed to submit contact information: " + sanitizeToText(String(errorData.reason ?? "")),
                    });
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