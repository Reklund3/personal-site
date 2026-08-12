import React, { createContext, useContext, useMemo, useState, type ReactNode } from 'react';

interface SectionNavHeightValue {
  navHeight: number;
  setNavHeight: (height: number) => void;
}

const SectionNavHeightContext = createContext<SectionNavHeightValue>({
  navHeight: 0,
  setNavHeight: () => {},
});

export function SectionNavHeightProvider({ children }: { children: ReactNode }) {
  const [navHeight, setNavHeight] = useState(0);
  // Memoize: an inline object literal changes identity every render and re-renders
  // every consumer, including sections that read scrollMarginTop on each parent update.
  const value = useMemo(() => ({ navHeight, setNavHeight }), [navHeight]);
  return <SectionNavHeightContext.Provider value={value}>{children}</SectionNavHeightContext.Provider>;
}

export const useSectionNavHeight = () => useContext(SectionNavHeightContext);
