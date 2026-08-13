import React, { createContext, useContext, useMemo, useState, type ReactNode } from 'react';

interface ContactDialogValue {
  open: boolean;
  openDialog: () => void;
  closeDialog: () => void;
}

const ContactDialogContext = createContext<ContactDialogValue>({
  open: false,
  openDialog: () => {},
  closeDialog: () => {},
});

export function ContactDialogProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);

  const openDialog = () => setOpen(true);
  const closeDialog = () => setOpen(false);

  // Memoize to prevent unnecessary re-renders of consumers
  const value = useMemo(() => ({ open, openDialog, closeDialog }), [open]);

  return <ContactDialogContext.Provider value={value}>{children}</ContactDialogContext.Provider>;
}

export const useContactDialog = () => useContext(ContactDialogContext);
