import { useEffect, useState } from 'react';
import {
  Box,
  Check,
  ChevronDown,
  Heart,
  Image,
  LoaderCircle,
  RotateCcw,
  SlidersHorizontal,
  Sparkles,
  X,
} from 'lucide-react';
import { CATALOG, hexTint, tintHex, toTexture, type MaterialEntry, type Texture } from '../../shared/catalog';
import { api, post, type MapUrls } from '../api';
import CubePreview from './CubePreview';
export const staticMaps = (entry: MaterialEntry): MapUrls => ({
  albedo: `/previews/${entry.name}.png`,
  normal: `/previews/${entry.name}_n.png`,
  specular: `/previews/${entry.name}_s.png`,
});
interface Props {
  entry: MaterialEntry;
  selected: boolean;
  favorite: boolean;
  seed: number;
  onClose: () => void;
  onSelect: () => void;
  onFavorite: () => void;
  onSave: (texture: Partial<Texture>) => void;
  onReset: () => void;
  notify: (text: string, error?: boolean) => void;
}
export default function Inspector({
  entry,
  selected,
  favorite,
  seed,
  onClose,
  onSelect,
  onFavorite,
  onSave,
  onReset,
  notify,
}: Props) {
  const [draft, setDraft] = useState(entry);
  const [maps, setMaps] = useState<MapUrls>(() =>
    entry.custom ? { albedo: '', normal: '', specular: '' } : staticMaps(entry),
  );
  const [mode, setMode] = useState<'cube' | 'texture'>('cube');
  const [channel, setChannel] = useState<keyof MapUrls>('albedo');
  const [advanced, setAdvanced] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const signature = JSON.stringify(toTexture(entry));
  const assetSignature = JSON.stringify(entry.assets);
  const savedSignature = JSON.stringify(entry);
  // Selection/favorite changes create a new entry object; preserve an unsaved draft.
  useEffect(() => {
    setDraft(entry);
    setError('');
  }, [savedSignature]);
  useEffect(() => {
    const original = CATALOG.find((c) => c.name === entry.name);
    if (!entry.custom && seed === 128042 && original && JSON.stringify(toTexture(original)) === signature) {
      setMaps(staticMaps(entry));
      return;
    }
    let cancelled = false;
    setBusy(true);
    setError('');
    post<MapUrls>('/previews', {
      texture: toTexture(entry),
      seed,
      assets: entry.assets,
      normal_y: entry.normal_y || 'directx',
    })
      .then((result) => {
        if (!cancelled) setMaps(result);
      })
      .catch((e) => {
        if (!cancelled) setError(e.message);
      })
      .finally(() => {
        if (!cancelled) setBusy(false);
      });
    return () => {
      cancelled = true;
    };
  }, [entry.name, signature, assetSignature, seed]);
  const dirty = JSON.stringify(draft) !== JSON.stringify(entry);
  const original = CATALOG.find((c) => c.name === entry.name);
  const hasSavedEdit = Boolean(original && JSON.stringify(toTexture(original)) !== signature);
  const update = (patch: Partial<Texture>) => setDraft((v) => ({ ...v, ...patch }));
  const apply = async () => {
    setBusy(true);
    setError('');
    try {
      const result = await post<MapUrls>('/previews', {
        texture: toTexture(draft),
        seed,
        assets: entry.assets,
        normal_y: entry.normal_y || 'directx',
      });
      setMaps(result);
      onSave(toTexture(draft));
      notify(`${entry.label} updated. Preview and pack settings are saved.`);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };
  return (
    <aside className="inspector" aria-label="Material inspector">
      <div className="inspector-top">
        <span>
          <SlidersHorizontal size={15} /> Material inspector
        </span>
        <button className="icon-button small" onClick={onClose} aria-label="Close inspector">
          <X size={17} />
        </button>
      </div>
      <div className="inspector-body">
        <div className="inspector-title">
          <div>
            <span className="eyebrow">{entry.custom ? 'YOUR MATERIAL' : 'SYNTHETIC COLLECTION'}</span>
            <h2>{entry.label}</h2>
          </div>
          <button
            className={`icon-button ${favorite ? 'is-favorite' : ''}`}
            onClick={onFavorite}
            aria-label={favorite ? 'Remove from favorites' : 'Add to favorites'}
            aria-pressed={favorite}
          >
            <Heart size={18} fill={favorite ? 'currentColor' : 'none'} />
          </button>
        </div>
        <div className="preview-mode">
          <button className={mode === 'cube' ? 'active' : ''} onClick={() => setMode('cube')}>
            <Box size={14} />
            3D block
          </button>
          <button className={mode === 'texture' ? 'active' : ''} onClick={() => setMode('texture')}>
            <Image size={14} />
            Texture
          </button>
        </div>
        <div className="inspector-preview">
          {maps.albedo &&
            (mode === 'cube' ? (
              <CubePreview maps={maps} entry={entry} />
            ) : (
              <div className="flat-preview checkerboard">
                <img src={maps[channel]} alt={`${entry.label} ${channel} map`} />
              </div>
            ))}
          {busy && (
            <div className="preview-loading">
              <LoaderCircle className="spin" size={24} />
              <span>Rendering on CPU…</span>
            </div>
          )}
        </div>
        <div className="map-tabs">
          {(['albedo', 'normal', 'specular'] as const).map((c) => (
            <button
              key={c}
              className={channel === c ? 'active' : ''}
              onClick={() => {
                setChannel(c);
                setMode('texture');
              }}
            >
              {c === 'albedo' ? 'Albedo' : c === 'normal' ? 'Normal' : 'Specular'}
              <span className={`channel-dot ${c}`} />
            </button>
          ))}
        </div>
        <p className="material-description">{entry.description}</p>
        <div className="property-heading">
          <h3>Surface properties</h3>
          <span>LabPBR 1.3</span>
        </div>
        <label className="color-property">
          <span>Base tint</span>
          <div>
            <span className="mono">{tintHex(draft.tint).toUpperCase()}</span>
            <input
              type="color"
              aria-label="Base tint"
              value={tintHex(draft.tint)}
              onChange={(e) => update({ tint: hexTint(e.target.value) })}
            />
          </div>
        </label>
        <label className="slider-property">
          <span>
            Roughness <b>{draft.roughness_scale.toFixed(2)}</b>
          </span>
          <input
            aria-label="Roughness"
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={draft.roughness_scale}
            onChange={(e) => update({ roughness_scale: +e.target.value })}
            style={{ '--range': `${draft.roughness_scale * 100}%` } as React.CSSProperties}
          />
          <div>
            <span>Polished</span>
            <span>Rough</span>
          </div>
        </label>
        <label className="slider-property">
          <span>
            Normal strength <b>{draft.normal_strength.toFixed(1)}</b>
          </span>
          <input
            aria-label="Normal strength"
            type="range"
            min="0"
            max="8"
            step="0.1"
            value={draft.normal_strength}
            onChange={(e) => update({ normal_strength: +e.target.value })}
            style={{ '--range': `${(draft.normal_strength / 8) * 100}%` } as React.CSSProperties}
          />
          <div>
            <span>Subtle</span>
            <span>Pronounced</span>
          </div>
        </label>
        <button className="advanced-toggle" onClick={() => setAdvanced((v) => !v)} aria-expanded={advanced}>
          Advanced settings
          <ChevronDown size={15} className={advanced ? 'rotate' : ''} />
        </button>
        {advanced && (
          <div className="advanced-fields">
            <label>
              F0 / metal ID
              <select
                aria-label="F0 or metal ID"
                value={draft.f0}
                onChange={(e) => update({ f0: +e.target.value })}
              >
                <option value={10}>Dielectric · 10</option>
                <option value={43}>Gem · 43</option>
                <option value={230}>Iron · 230</option>
                <option value={231}>Gold · 231</option>
                <option value={232}>Copper · 232</option>
                <option value={233}>Aluminum · 233</option>
                <option value={234}>Chromium · 234</option>
                <option value={235}>Zinc · 235</option>
                <option value={236}>Tin · 236</option>
                <option value={237}>Platinum · 237</option>
                <option value={255}>Custom metal · 255</option>
                {![10, 43, 230, 231, 232, 233, 234, 235, 236, 237, 255].includes(draft.f0) && (
                  <option value={draft.f0}>Current · {draft.f0}</option>
                )}
              </select>
            </label>
            <label className="slider-property">
              <span>
                Porosity <b>{draft.porosity}</b>
              </span>
              <input
                aria-label="Porosity"
                type="range"
                min="0"
                max="64"
                value={draft.porosity}
                onChange={(e) => update({ porosity: +e.target.value })}
                style={{ '--range': `${(draft.porosity / 64) * 100}%` } as React.CSSProperties}
              />
            </label>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={draft.emissive !== 255}
                onChange={(e) => update({ emissive: e.target.checked ? 128 : 255 })}
              />
              Enable emission
            </label>
            {draft.emissive !== 255 && (
              <label className="slider-property">
                <span>
                  Emission <b>{draft.emissive}</b>
                </span>
                <input
                  aria-label="Emission"
                  type="range"
                  min="0"
                  max="254"
                  value={draft.emissive}
                  onChange={(e) => update({ emissive: +e.target.value })}
                  style={{ '--range': `${(draft.emissive / 254) * 100}%` } as React.CSSProperties}
                />
              </label>
            )}
            <p>255 in the emission channel means no emission, not full brightness.</p>
          </div>
        )}
        {error && (
          <div className="inline-error" role="alert">
            {error}
          </div>
        )}
        <div className="inspector-actions">
          <button className="button primary" disabled={!dirty || busy} onClick={apply}>
            {busy ? <LoaderCircle className="spin" size={16} /> : <Sparkles size={16} />}
            {busy ? 'Generating…' : 'Apply & preview'}
          </button>
          <button
            className="icon-button reset-button"
            title="Reset material to its original settings"
            aria-label="Reset material"
            disabled={busy || (!dirty && !hasSavedEdit)}
            onClick={() => {
              setDraft(entry);
              onReset();
              setMaps(entry.custom ? maps : staticMaps(CATALOG.find((c) => c.name === entry.name) || entry));
              notify('Material reset to its original settings.');
            }}
          >
            <RotateCcw size={16} />
          </button>
        </div>
        <button
          className={`include-material ${selected ? 'included' : ''}`}
          onClick={onSelect}
          aria-pressed={selected}
        >
          <span className="selection-check">{selected && <Check size={12} strokeWidth={3} />}</span>
          {selected ? 'Included in resource pack' : 'Include in resource pack'}
        </button>
        <p className="inspector-footnote">
          {entry.custom
            ? 'Your source images stay on this server.'
            : 'Original procedural demo · not a scanned material.'}
        </p>
      </div>
    </aside>
  );
}
