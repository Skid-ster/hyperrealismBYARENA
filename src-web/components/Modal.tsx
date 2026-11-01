import { useEffect, useRef, type ReactNode } from 'react';
import { X } from 'lucide-react';
export default function Modal({
  title,
  subtitle,
  children,
  onClose,
  wide = false,
}: {
  title: string;
  subtitle?: string;
  children: ReactNode;
  onClose: () => void;
  wide?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const closeRef = useRef(onClose);
  closeRef.current = onClose;
  useEffect(() => {
    const previous = document.activeElement as HTMLElement;
    const overflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    const timer = window.setTimeout(
      () =>
        ref.current
          ?.querySelector<HTMLElement>('input:not([type="file"]), button, select, [tabindex="0"]')
          ?.focus(),
      30,
    );
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') closeRef.current();
      if (e.key === 'Tab') {
        const elements = ref.current?.querySelectorAll<HTMLElement>(
          'button:not([disabled]), input:not([disabled]):not([type="file"]), select:not([disabled]), a[href], [tabindex="0"]',
        );
        if (!elements?.length) return;
        const first = elements[0];
        const last = elements[elements.length - 1];
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener('keydown', handler);
    return () => {
      clearTimeout(timer);
      document.body.style.overflow = overflow;
      document.removeEventListener('keydown', handler);
      previous?.focus();
    };
  }, []);
  return (
    <div
      className="modal-backdrop"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        className={`modal ${wide ? 'modal-wide' : ''}`}
        ref={ref}
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
      >
        <div className="modal-heading">
          <div>
            <h2 id="dialog-title">{title}</h2>
            {subtitle && <p>{subtitle}</p>}
          </div>
          <button className="icon-button" onClick={onClose} aria-label="Close dialog">
            <X size={20} />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}
