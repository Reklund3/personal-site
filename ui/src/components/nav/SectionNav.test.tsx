import React from 'react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { createMemoryRouter, RouterProvider } from 'react-router-dom';
import { ThemeProvider } from '@mui/material/styles';
import theme from '../../theme';
import { SectionNavHeightProvider, useSectionNavHeight } from '../../context/SectionNavHeightContext';
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

  // Run the rAF callback synchronously so assertions need no timer flushing. The two
  // lifecycle tests below override this with a deferring stub, because a frame that
  // has already run cannot demonstrate anything about cancelling a pending one.
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
    cb(0);
    return 0;
  });
  // jsdom does supply cancelAnimationFrame, but it cancels ITS scheduler, not the stub
  // above — so the pair has to be replaced together or cleanup would hand a real
  // canceller a handle it never issued.
  vi.stubGlobal('cancelAnimationFrame', vi.fn());

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

// `extra` renders inside SectionNavHeightProvider, so a probe passed here can observe
// the same navHeight the real Section components consume.
function renderSectionNav(initialPath: string, extra?: React.ReactNode) {
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

  const view = render(
    <ThemeProvider theme={theme}>
      <SectionNavHeightProvider>
        {extra}
        <RouterProvider router={router} />
      </SectionNavHeightProvider>
    </ThemeProvider>
  );

  return { router, unmount: view.unmount };
}

describe('SectionNav', () => {
  it('scrolls on a genuine deep link even after a same-section tab click (#24 regression)', async () => {
    const { router } = renderSectionNav('/skills');
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
    const { router } = renderSectionNav('/');
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
    const { router } = renderSectionNav('/');

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

  describe('pending frame lifecycle', () => {
    /**
     * Replaces the synchronous stub from beforeEach with one that queues instead of
     * running, which is what a real frame does — it fires after React has finished
     * committing, not inline in the effect that scheduled it.
     */
    function deferFrames() {
      const pending = new Map<number, FrameRequestCallback>();
      let nextHandle = 1;
      vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
        const handle = nextHandle++;
        pending.set(handle, cb);
        return handle;
      });
      vi.stubGlobal('cancelAnimationFrame', (handle: number) => {
        pending.delete(handle);
      });
      return {
        pending,
        flush: () => {
          const queued = [...pending.values()];
          pending.clear();
          queued.forEach((cb) => cb(0));
        },
      };
    }

    it('cancels the pending deep-link frame on unmount', () => {
      const frames = deferFrames();

      const { unmount } = renderSectionNav('/skills');
      // The deep link queued a scroll that has not run yet.
      expect(frames.pending.size).toBe(1);

      unmount();

      // Cleanup cancelled it, so nothing is left to fire against a torn-down tree.
      expect(frames.pending.size).toBe(0);
      frames.flush();
      expect(scrollIntoViewMock).not.toHaveBeenCalled();
    });

    it('does not scroll to the section it just left when a navigation lands in the same frame', async () => {
      const frames = deferFrames();

      const { router } = renderSectionNav('/skills');
      // /skills queued its scroll but the frame has not run.
      expect(frames.pending.size).toBe(1);

      // Navigate before that frame fires. The effect re-runs, and its cleanup must
      // drop the stale /skills frame rather than let it scroll somewhere the user
      // has already navigated away from.
      await act(async () => {
        await router.navigate('/experience');
      });

      frames.flush();

      expect(scrollIntoViewMock).toHaveBeenCalledTimes(1);
      expect(scrollIntoViewMock.mock.contexts[0]).toBe(document.getElementById('experience'));
    });

    /**
     * The #24 secondary defect: Section applies scrollMarginTop: navHeight, so a scroll
     * firing while navHeight is still 0 lands the section under the sticky bar.
     *
     * What this test locks in: the height reaches context from a direct measurement of
     * the nav, so it is available by the time a frame runs. MockResizeObserver never
     * invokes its callback, so a version that only publishes from the observer reads 0.
     *
     * What it CANNOT lock in: that the measurement happens in a layout effect rather
     * than a passive one. jsdom never paints, so "before paint" has no meaning here and
     * swapping useLayoutEffect for useEffect keeps this green — verified by mutation.
     * That half of the fix rests on the manual browser check in the PR.
     */
    it('publishes a direct nav measurement, not one that waits on the ResizeObserver', () => {
      // Only the sticky container reports a height. A prototype-wide constant would let
      // a measurement of the wrong node (document.body, say) still read 48 and pass.
      vi.spyOn(HTMLElement.prototype, 'offsetHeight', 'get').mockImplementation(function (
        this: HTMLElement
      ) {
        return this.firstElementChild?.getAttribute('role') === 'navigation' ? 48 : 0;
      });
      const frames = deferFrames();

      let navHeightAtScrollTime: number | null = null;
      let latestNavHeight = 0;
      scrollIntoViewMock.mockImplementation(() => {
        navHeightAtScrollTime = latestNavHeight;
      });

      function NavHeightProbe() {
        latestNavHeight = useSectionNavHeight().navHeight;
        return null;
      }

      renderSectionNav('/skills', <NavHeightProbe />);
      frames.flush();

      expect(navHeightAtScrollTime).toBe(48);
    });
  });
});
