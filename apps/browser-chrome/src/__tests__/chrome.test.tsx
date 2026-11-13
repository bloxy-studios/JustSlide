import { render, screen, within, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BridgeContext, type ChromeBridge } from '../bridge/bridge';
import { createFixtureBridge } from '../bridge/fixture';
import { SlideWindow } from '../components/SlideWindow';
import { displayUrl } from '../components/AddressSearchBar';
import { contrast, tintFor, FIGMA_EXAMPLE_BACKDROP } from '../theme/slide-theme';

function setup(variant: 'desktop' | 'compact' = 'desktop') {
  const real = createFixtureBridge(variant);
  const calls: Array<[string, unknown]> = [];
  const bridge: ChromeBridge = { ...real, kind: real.kind, getSnapshot: real.getSnapshot, subscribe: real.subscribe, reportContentBounds: () => {},
    execute: (c, a) => { calls.push([c, a]); return real.execute(c, a); } };
  render(<BridgeContext.Provider value={bridge}><SlideWindow preview /></BridgeContext.Provider>);
  return { calls, bridge };
}

describe('information architecture (Figma 1:6)', () => {
  it('keeps website favorites and workspace switchers semantically distinct', () => {
    setup();
    const favs = screen.getByRole('list', { name: 'Website favorites' });
    expect(within(favs).getByRole('button', { name: 'Open ChatGPT website' })).toBeTruthy();
    expect(within(favs).getByRole('button', { name: 'Open Claude website' })).toBeTruthy();
    expect(within(favs).getAllByRole('button')).toHaveLength(13);
    const dock = screen.getByRole('list', { name: 'Workspaces' });
    for (const n of ['Personal', 'Codex', 'Claude Code', 'ChatGPT']) expect(within(dock).getByRole('button', { name: new RegExp(`^Switch to ${n} workspace`) })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Create new workspace' })).toBeTruthy();
    expect(within(dock).queryByRole('button', { name: /website/ })).toBeNull();
  });

  it('orders the sidebar top to bottom as specified', () => {
    setup();
    const nav = screen.getByRole('navigation', { name: 'Browser sidebar' });
    const order = [...nav.querySelectorAll('[data-figma]')].map((e) => e.getAttribute('data-figma'));
    const want = ['6:17', '7:46', '7:87', '25:4', '12:139', '20:82', '12:290'];
    expect(order.filter((o) => want.includes(o!))).toEqual(want);
  });

  it('shows the real active tab, its host in the address bar, and selects tabs via the core', async () => {
    const { calls } = setup();
    const selected = screen.getByRole('button', { name: 'Slide - Figma' });
    expect(selected.getAttribute('aria-current')).toBe('page');
    const addr = screen.getByRole('textbox', { name: 'Address and search' }) as HTMLInputElement;
    expect(addr.value).toBe('figma.com');
    await userEvent.click(screen.getByRole('button', { name: 'Icons - Lucide' }));
    expect(calls).toContainEqual(['tabs.activate', expect.objectContaining({ tabId: expect.stringMatching(/^tab_/) })]);
    expect(screen.getByRole('button', { name: 'Icons - Lucide' }).getAttribute('aria-current')).toBe('page');
    expect(addr.value).toBe('lucide.dev');
  });

  it('address bar edits the full URL and never overwrites typing', async () => {
    const { calls } = setup();
    const addr = screen.getByRole('textbox', { name: 'Address and search' }) as HTMLInputElement;
    await userEvent.click(addr);
    expect(addr.value).toBe('https://www.figma.com/design/nw47hqMLTtmBMksJ4BnJKb/Slide');
    await userEvent.clear(addr);
    await userEvent.type(addr, 'rust ownership{Enter}');
    expect(calls).toContainEqual(['page.navigate', expect.objectContaining({ url: 'rust ownership' })]);
  });

  it('pinned section collapses through the core and reports its count', async () => {
    const { calls } = setup();
    const trigger = screen.getByRole('button', { name: /Pinned/ });
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
    expect(within(trigger).getByLabelText('5 pinned')).toBeTruthy();
    await userEvent.click(trigger);
    expect(calls).toContainEqual(['ui.setPinnedCollapsed', { collapsed: true }]);
    expect(screen.getByRole('button', { name: /Pinned/ }).getAttribute('aria-expanded')).toBe('false');
    expect(screen.queryByRole('button', { name: 'Ali express' })).toBeNull();
  });
});

describe('agent activity (Figma 35:13)', () => {
  it('is absent when no agent is working', () => {
    setup('desktop');
    expect(screen.queryByRole('region', { name: /activity/ })).toBeNull();
    expect(screen.queryByTestId('activity-dot')).toBeNull();
  });

  it('shows a minimal card and a dock indicator only for a working agent; pause is a real command', async () => {
    const { calls } = setup('compact');
    const card = screen.getByRole('region', { name: 'Claude Code activity' });
    expect(card.textContent).toContain('Claude Code · Working');
    expect(card.textContent).toContain('3 tabs in its workspace');
    expect(screen.getAllByTestId('activity-dot')).toHaveLength(1);
    await userEvent.click(within(card).getByRole('button', { name: 'Pause Claude Code' }));
    expect(calls).toContainEqual(['agents.pause', { grantId: 'grant_fixture' }]);
    expect(within(screen.getByRole('region', { name: 'Claude Code activity' })).getByRole('button', { name: 'Resume Claude Code' })).toBeTruthy();
    // Personal remains the selected workspace while the agent works.
    expect(screen.getByRole('button', { name: /^Switch to Personal workspace/ }).getAttribute('aria-pressed')).toBe('true');
  });
});

describe('Slide theme layer', () => {
  it('reproduces the Figma inks exactly at the example backdrop', () => {
    expect(tintFor(FIGMA_EXAMPLE_BACKDROP)['--ds-text']).toBe('#0E1B1B');
    expect(tintFor('#acfff7')['--ds-border-focused']).toBe('#10574F');
  });
  it('re-tints for other backdrops while keeping text contrast', () => {
    for (const bd of ['#FFD6E7', '#E4E4E7', '#FFF3B0', '#C7D2FE']) {
      const t = tintFor(bd);
      expect(t['--ds-text']).not.toBe('#0E1B1B');
      expect(contrast(t['--ds-text'], bd)).toBeGreaterThan(7);
      expect(contrast(t['--ds-text'], '#FFFFFF')).toBeGreaterThan(7);
      expect(contrast(t['--ds-border-focused'], '#FFFFFF')).toBeGreaterThan(3);
    }
  });
  it('keeps primary text AA+ on the example backdrop and surfaces', () => {
    expect(contrast('#0E1B1B', FIGMA_EXAMPLE_BACKDROP)).toBeGreaterThan(7);
    expect(contrast('#304B48', '#FFFFFF')).toBeGreaterThan(4.5);
  });
});

describe('address formatting', () => {
  it('shows hosts without www at rest', () => {
    expect(displayUrl('https://www.figma.com/design/x')).toBe('figma.com');
    expect(displayUrl('about:blank')).toBe('');
  });
});
