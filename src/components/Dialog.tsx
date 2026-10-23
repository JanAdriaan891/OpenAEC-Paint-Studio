import { useEffect, useRef, type ReactNode } from 'react';
export default function Dialog({ title, open, close, children }: { title: string; open: boolean; close: () => void; children: ReactNode }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => { if (open && !ref.current?.open) ref.current?.showModal(); if (!open) ref.current?.close(); }, [open]);
  return <dialog ref={ref} onCancel={close} aria-label={title}><header><h2>{title}</h2><button onClick={close} aria-label="Close">×</button></header><div className="dialog-body">{children}</div></dialog>;
}
