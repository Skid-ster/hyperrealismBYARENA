import { useState } from 'react';
import { Cpu, Info, Save } from 'lucide-react';
import Modal from './Modal';
import { settingsSchema } from '../../shared/validation';
import type { Settings } from '../../shared/catalog';
export default function SettingsDialog({
  settings,
  onSave,
  onClose,
}: {
  settings: Settings;
  onSave: (s: Settings) => void;
  onClose: () => void;
}) {
  const [draft, setDraft] = useState(settings);
  const [error, setError] = useState('');
  const patch = (v: Partial<Settings>) => setDraft((s) => ({ ...s, ...v }));
  return (
    <Modal
      title="Pack settings"
      subtitle="A few details before your world gets an upgrade."
      onClose={onClose}
    >
      <form
        className="settings-form"
        onSubmit={(e) => {
          e.preventDefault();
          const parsed = settingsSchema.safeParse(draft);
          if (!parsed.success) setError(parsed.error.issues[0].message);
          else onSave(parsed.data);
        }}
      >
        <label>
          Pack name
          <input
            value={draft.packName}
            maxLength={60}
            onChange={(e) => patch({ packName: e.target.value })}
            required
          />
        </label>
        <div className="form-two">
          <label>
            Output resolution
            <select
              value={draft.resolution}
              onChange={(e) => patch({ resolution: +e.target.value as Settings['resolution'] })}
            >
              <option value={64}>64 × 64 · Lightweight</option>
              <option value={128}>128 × 128 · Recommended</option>
              <option value={256}>256 × 256 · High detail</option>
            </select>
          </label>
          <label>
            Generation seed
            <input
              type="number"
              min={0}
              max={4294967295}
              value={draft.seed}
              onChange={(e) => patch({ seed: +e.target.value })}
              required
            />
          </label>
        </div>
        <p className="form-hint">The same seed, source images, and settings produce the same pack.</p>
        <div className="settings-section">
          <h3>PBR map channels</h3>
          <label className="switch-row">
            <span>
              <strong>Normal & height maps</strong>
              <small>Surface detail, ambient occlusion, and height (_n)</small>
            </span>
            <input
              className="switch"
              type="checkbox"
              checked={draft.includeNormal}
              onChange={(e) => patch({ includeNormal: e.target.checked })}
            />
          </label>
          <label className="switch-row">
            <span>
              <strong>Specular maps</strong>
              <small>Roughness, reflectance, porosity, and emission (_s)</small>
            </span>
            <input
              className="switch"
              type="checkbox"
              checked={draft.includeSpecular}
              onChange={(e) => patch({ includeSpecular: e.target.checked })}
            />
          </label>
        </div>
        <div className="settings-section">
          <h3>
            <Cpu size={16} /> CPU performance
          </h3>
          <label className="slider-property">
            <span>
              Worker threads <b>{draft.threads}</b>
            </span>
            <input
              type="range"
              min={1}
              max={16}
              value={draft.threads}
              onChange={(e) => patch({ threads: +e.target.value })}
              style={{ '--range': `${((draft.threads - 1) / 15) * 100}%` } as React.CSSProperties}
            />
            <div>
              <span>Lower system usage</span>
              <span>Faster generation</span>
            </div>
          </label>
          <p className="form-hint">
            The operating system schedules workers. No core pinning or GPU acceleration.
          </p>
        </div>
        <div className="notice">
          <Info size={18} />
          <p>
            Target: <strong>Minecraft Java 26.3</strong> · pack format 97.1. A LabPBR-compatible shader is
            required. In-game compatibility has not been verified.
          </p>
        </div>
        {error && (
          <div className="inline-error" role="alert">
            {error}
          </div>
        )}
        <div className="modal-actions">
          <button type="button" className="button secondary" onClick={onClose}>
            Cancel
          </button>
          <button className="button primary">
            <Save size={16} />
            Save settings
          </button>
        </div>
      </form>
    </Modal>
  );
}
