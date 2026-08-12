import React, { createContext, useContext, useMemo, useState, type ReactNode } from 'react';

interface AppBarHeightValue {
  appBarHeight: number;
  setAppBarHeight: (height: number) => void;
}

const AppBarHeightContext = createContext<AppBarHeightValue>({
  appBarHeight: 0,
  setAppBarHeight: () => {},
});

export function AppBarHeightProvider({ children }: { children: ReactNode }) {
  const [appBarHeight, setAppBarHeight] = useState(0);
  // Memoize: an inline object literal changes identity every render and re-renders
  // every consumer, including the sticky nav on each scroll-driven parent update.
  const value = useMemo(() => ({ appBarHeight, setAppBarHeight }), [appBarHeight]);
  return <AppBarHeightContext.Provider value={value}>{children}</AppBarHeightContext.Provider>;
}

export const useAppBarHeight = () => useContext(AppBarHeightContext);
