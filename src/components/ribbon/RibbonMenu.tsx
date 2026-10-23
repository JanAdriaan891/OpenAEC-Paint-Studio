import { useEffect, useRef, type ReactNode, type ComponentType } from 'react';

/** Fixed-position menus stay visible when the ribbon scrolls on small windows. */
export default function RibbonMenu({ label, icon: Icon, children }: {
  label: string; icon: ComponentType<{ size?: number }>; children: ReactNode;
}) {
  const details = useRef<HTMLDetailsElement>(null);
  useEffect(() => {
    const closeOutside = (event: globalThis.PointerEvent) => {
      if (!details.current?.contains(event.target as Node)) details.current?.removeAttribute('open');
    };
    const close = () => details.current?.removeAttribute('open');
    const scroll = (event: Event) => { if (!details.current?.contains(event.target as Node)) close(); };
    const escape = (event: KeyboardEvent) => { if (event.key === 'Escape' && details.current?.open) { close(); details.current.querySelector('summary')?.focus(); } };
    document.addEventListener('pointerdown', closeOutside);
    document.addEventListener('keydown', escape);
    window.addEventListener('resize', close);
    document.addEventListener('scroll', scroll, true);
    return () => { document.removeEventListener('pointerdown', closeOutside); document.removeEventListener('keydown', escape); window.removeEventListener('resize', close); document.removeEventListener('scroll', scroll, true); };
  }, []);
  return <details ref={details} className="ribbon-menu" onToggle={() => {
    const element = details.current;
    if (!element?.open) return;
    document.querySelectorAll<HTMLDetailsElement>('.ribbon-menu[open]').forEach(other => { if (other !== element) other.open = false; });
    const box = element.querySelector('summary')!.getBoundingClientRect();
    const panel = element.querySelector<HTMLElement>('.ribbon-menu-panel')!;
    panel.style.left = `${Math.max(8, Math.min(box.left, window.innerWidth - panel.offsetWidth - 8))}px`;
    panel.style.top = `${box.bottom + 6}px`;
    panel.style.maxHeight = `${Math.max(80, window.innerHeight - box.bottom - 20)}px`;
  }}>
    <summary><Icon size={17}/><span>{label}</span><span className="menu-chevron">⌄</span></summary>
    <div className="ribbon-menu-panel" onClick={event => {
      if ((event.target as HTMLElement).closest('button')) details.current!.open = false;
    }}>{children}</div>
  </details>;
}
