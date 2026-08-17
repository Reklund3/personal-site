import React from 'react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { createMemoryRouter, RouterProvider } from 'react-router-dom';
import { ThemeProvider } from '@mui/material/styles';
import theme from '../../theme';
import { SectionNavHeightProvider } from '../../context/SectionNavHeightContext';
import SectionNav from './SectionNav';

const SECTION_IDS = ['about', 'skills', 'experience', 'education', 'portfolio'];

let scrollIntoViewMock: ReturnType<typeof vi.fn>;

beforeEach(() => {
  // jsdom does not implement scrollIntoView at all — stub it on the prototype so
  // every element picks it up, and keep the mock itself as the assertion target.
  scrollIntoViewMock = vi.fn();
  Element.prototype.scrollIntoView = scrollIntoViewMock as unknown as typeof Element.prototype.scrollIntoView;

  // jsdom does not implement ResizeObserver. MUI's scrollable Tabs uses one too,
  // so this needs to exist even though its callback is never exercised here.
  class MockResizeObserver {
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
  }
  vi.stubGlobal('ResizeObserver', MockResizeObserver);

  // jsdom does not implement IntersectionObserver. The scroll-spy effect
  // constructs one on every mount.
  class MockIntersectionObserver {
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
  }
  vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);

  // Run the rAF callback synchronously so assertions need no timer flushing.
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
    cb(0);
    return 0;
  });

  // jsdom does not implement window.matchMedia. handleTabChange reads it on
  // every plain click to decide smooth vs. instant scrolling.
  vi.stubGlobal(
    'matchMedia',
    vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

function renderSectionNav(initialPath: string) {
  const router = createMemoryRouter(
    [
      {
        path: '*',
        element: (
          <>
            <SectionNav />
            {SECTION_IDS.map((id) => (
              <div id={id} key={id} />
            ))}
          </>
        ),
      },
    ],
    { initialEntries: [initialPath] }
  );

  render(
    <ThemeProvider theme={theme}>
      <SectionNavHeightProvider>
        <RouterProvider router={router} />
      </SectionNavHeightProvider>
    </ThemeProvider>
  );

  return router;
}

describe('SectionNav', () => {
  it('scrolls on a genuine deep link even after a same-section tab click (#24 regression)', async () => {
    const router = renderSectionNav('/skills');
    // Mount itself deep-links to #skills — isolate what follows.
    scrollIntoViewMock.mockClear();

    // Same-section click: already on /skills, so navigate() replaces to the same
    // path. Before the fix this left programmaticScrollRef stranded on 'skills'.
    const skillsTab = screen.getByRole('link', { name: 'Skills' });
    await act(async () => {
      fireEvent.click(skillsTab);
    });
    expect(scrollIntoViewMock).toHaveBeenCalledTimes(1);
    expect(scrollIntoViewMock).toHaveBeenLastCalledWith({ behavior: 'smooth', block: 'start' });

    // The next genuine navigation to that same section — e.g. Back then Forward —
    // must still scroll. Before the fix the stranded flag swallowed this.
    await act(async () => {
      await router.navigate('/skills');
    });

    expect(scrollIntoViewMock).toHaveBeenCalledTimes(2);
    expect(scrollIntoViewMock).toHaveBeenLastCalledWith({ behavior: 'auto', block: 'start' });
  });

  it('scrolls exactly once on mount for a direct deep link', () => {
    renderSectionNav('/skills');

    expect(scrollIntoViewMock).toHaveBeenCalledTimes(1);
    expect(scrollIntoViewMock).toHaveBeenCalledWith({ behavior: 'auto', block: 'start' });
  });

  it('scrolls smoothly on a tab click from / and does not double-scroll', async () => {
    const router = renderSectionNav('/');
    // '/' needs no scroll on mount.
    expect(scrollIntoViewMock).not.toHaveBeenCalled();

    const skillsTab = screen.getByRole('link', { name: 'Skills' });
    await act(async () => {
      fireEvent.click(skillsTab);
    });

    expect(scrollIntoViewMock).toHaveBeenCalledTimes(1);
    expect(scrollIntoViewMock).toHaveBeenLastCalledWith({ behavior: 'smooth', block: 'start' });
    expect(router.state.location.pathname).toBe('/skills');
  });

  it('leaves modified clicks to the browser', async () => {
    const router = renderSectionNav('/');

    const skillsTab = screen.getByRole('link', { name: 'Skills' });
    let notPrevented = false;
    await act(async () => {
      notPrevented = fireEvent.click(skillsTab, { metaKey: true });
    });

    expect(scrollIntoViewMock).not.toHaveBeenCalled();
    // fireEvent's dispatchEvent-derived return value is true when the event was
    // NOT cancelled — i.e. preventDefault() was never called.
    expect(notPrevented).toBe(true);
    expect(router.state.location.pathname).toBe('/');
  });

  it.each(['/open-source', '/projects'])('scrolls to #portfolio for the legacy path %s', (path) => {
    renderSectionNav(path);

    const portfolioEl = document.getElementById('portfolio');
    expect(scrollIntoViewMock).toHaveBeenCalledTimes(1);
    expect(scrollIntoViewMock).toHaveBeenCalledWith({ behavior: 'auto', block: 'start' });
    expect(scrollIntoViewMock.mock.contexts[0]).toBe(portfolioEl);
  });
});
