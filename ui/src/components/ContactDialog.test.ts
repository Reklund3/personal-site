import { describe, it, expect } from 'vitest';
import { validateName, validateEmail, validateMessage, findHtmlTags } from './ContactDialog';

describe('ContactDialog validation logic', () => {
    describe('validateName', () => {
        it('returns error for empty name', () => {
            expect(validateName('')).toBe('This field is required');
        });

        it('returns error for illegal characters', () => {
            expect(validateName('John <Doe>')).toContain('contains illegal characters');
        });

        it('returns error for name too long', () => {
            const longName = 'a'.repeat(257);
            expect(validateName(longName)).toContain('too many');
        });

        it('returns empty string for valid name', () => {
            expect(validateName('John Doe')).toBe('');
        });
    });

    describe('validateEmail', () => {
        it('returns error for empty email', () => {
            expect(validateEmail('')).toBe('This field is required');
        });

        it('returns error for email without @', () => {
            expect(validateEmail('johndoe')).toBe('Email address is missing an @');
        });

        it('returns error for email with multiple @', () => {
            expect(validateEmail('john@doe@example.com')).toBe('Email address has more than one @');
        });

        it('returns error for email without domain', () => {
            expect(validateEmail('john@')).toBe('Email address is missing the domain after the @');
        });

        it('returns error for email without dot in domain', () => {
            expect(validateEmail('john@doe')).toBe('Email domain is missing a dot, e.g. example.com');
        });

        it('returns error for email with spaces', () => {
            expect(validateEmail('john doe@example.com')).toBe('Email address cannot contain spaces');
        });

        it('returns error for invalid email format', () => {
            expect(validateEmail('john@example.')).toBe('Please enter a valid email address, e.g. name@example.com');
        });

        it('returns empty string for valid email', () => {
            expect(validateEmail('john.doe@example.com')).toBe('');
        });
    });

    describe('validateMessage', () => {
        it('returns error for empty message', () => {
            expect(validateMessage('')).toBe('This field is required');
        });

        it('returns error for message that is only HTML', () => {
            expect(validateMessage('<div></div>')).toBe('Your message is entirely HTML and would arrive empty. Please write it as plain text.');
        });

        it('returns error for message containing HTML tags', () => {
            expect(validateMessage('Hello <b>world</b>')).toContain('contains HTML tags: <b>');
        });

        it('returns error for message that is too long', () => {
            const longMessage = 'a'.repeat(1025);
            expect(validateMessage(longMessage)).toContain('too many');
        });

        it('returns empty string for valid message', () => {
            expect(validateMessage('Hello world!')).toBe('');
        });
    });

    describe('findHtmlTags', () => {
        it('finds tags in a string', () => {
            expect(findHtmlTags('<div><p>Hello</p></div>')).toEqual(['div', 'p']);
        });

        it('returns empty array for no tags', () => {
            expect(findHtmlTags('Hello world')).toEqual([]);
        });
    });
});
