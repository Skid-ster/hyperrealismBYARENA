import { useRef, useState } from 'react';
import { UploadCloud, FileImage, X, LoaderCircle, ArrowRight, ShieldCheck } from 'lucide-react';
import Modal from './Modal';
import { api } from '../api';
import { CATEGORIES, RECIPES, type MaterialEntry, type Category, type Recipe } from '../../shared/catalog';
import { textureName } from '../../shared/validation';
export default function ImportDialog({
  onClose,
  onImport,
  existingNames,
}: {
  onClose: () => void;
  onImport: (entry: MaterialEntry) => Promise<void>;
  existingNames: string[];
}) {
  const [label, setLabel] = useState('');
  const [name, setName] = useState('');
  const [recipe, setRecipe] = useState<Recipe>('stone');
  const [category, setCategory] = useState<Category>('Stone');
  const [normalY, setNormalY] = useState<'directx' | 'opengl'>('directx');
  const [files, setFiles] = useState<{ albedo?: File; normal?: File; roughness?: File }>({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [dragging, setDragging] = useState(false);
  const inputs = useRef<Record<string, HTMLInputElement | null>>({});
  const choose = (kind: 'albedo' | 'normal' | 'roughness', file?: File) => {
    if (!file) return;
    if (!file.name.toLowerCase().endsWith('.png') || file.size > 24 * 1024 * 1024) {
      setError('Choose a PNG smaller than 24 MB.');
      return;
    }
    setError('');
    setFiles((v) => ({ ...v, [kind]: file }));
  };
  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    const parsed = textureName.safeParse(name);
    if (!parsed.success) {
      setError(parsed.error.issues[0].message);
      return;
    }
    if (existingNames.includes(name)) {
      setError('A material with this filename already exists. Choose another name.');
      return;
    }
    if (!files.albedo) {
      setError('An albedo PNG is required.');
      return;
    }
    if (!label.trim()) {
      setError('Give your material a display name.');
      return;
    }
    setBusy(true);
    try {
      const assets: Record<string, string> = {};
      for (const [kind, file] of Object.entries(files)) {
        if (!file) continue;
        const form = new FormData();
        form.append('image', file);
        const result = await api<{ id: string }>(`/uploads`, { method: 'POST', body: form });
        assets[kind] = result.id;
      }
      await onImport({
        name,
        label: label.trim(),
        material: name,
        category,
        recipe,
        description: 'Your imported material, generated locally with the CPU engine.',
        custom: true,
        assets: assets as MaterialEntry['assets'],
        normal_y: normalY,
        tint: [255, 255, 255],
        f0: 10,
        porosity: 0,
        emissive: 255,
        roughness_scale: 0.8,
        normal_strength: 2.5,
      });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal
      title="Import a material"
      subtitle="Bring your own textures. We’ll take care of the PBR pipeline."
      onClose={() => {
        if (!busy) onClose();
      }}
      wide
    >
      <form onSubmit={submit} className="import-form">
        <div className="form-two">
          <label>
            Display name
            <input
              autoFocus
              placeholder="e.g. Weathered limestone"
              value={label}
              maxLength={80}
              onChange={(e) => {
                setLabel(e.target.value);
                setName(
                  e.target.value
                    .toLowerCase()
                    .replace(/[^a-z0-9]+/g, '_')
                    .replace(/^_|_$/g, '')
                    .slice(0, 80),
                );
              }}
              required
            />
          </label>
          <label>
            Texture filename
            <div className="input-suffix">
              <input
                placeholder="weathered_limestone"
                value={name}
                maxLength={80}
                onChange={(e) => setName(e.target.value)}
                required
              />
              <span>.png</span>
            </div>
          </label>
        </div>
        <div
          className={`albedo-drop ${dragging ? 'dragging' : ''}`}
          onDragOver={(e) => {
            e.preventDefault();
            setDragging(true);
          }}
          onDragLeave={() => setDragging(false)}
          onDrop={(e) => {
            e.preventDefault();
            setDragging(false);
            choose('albedo', e.dataTransfer.files[0]);
          }}
        >
          <UploadCloud size={30} />
          <strong>{files.albedo ? files.albedo.name : 'Drop your albedo texture here'}</strong>
          <span>PNG · up to 4096 × 4096 · 24 MB maximum</span>
          <button type="button" className="button secondary" onClick={() => inputs.current.albedo?.click()}>
            {files.albedo ? 'Choose a different image' : 'Browse files'}
          </button>
          <input
            ref={(el) => {
              inputs.current.albedo = el;
            }}
            className="sr-only"
            tabIndex={-1}
            type="file"
            accept=".png,image/png"
            aria-label="Albedo PNG file"
            onChange={(e) => choose('albedo', e.target.files?.[0])}
          />
        </div>
        <div className="form-two">
          {(['normal', 'roughness'] as const).map((kind) => (
            <div className="optional-upload" key={kind}>
              <div>
                <FileImage size={19} />
                <span>
                  <strong>{kind === 'normal' ? 'Normal map' : 'Roughness map'}</strong>
                  <small>{files[kind]?.name || 'Optional PNG'}</small>
                </span>
              </div>
              {files[kind] ? (
                <button
                  type="button"
                  className="icon-button"
                  aria-label={`Remove ${kind} map`}
                  onClick={() => setFiles((v) => ({ ...v, [kind]: undefined }))}
                >
                  <X size={16} />
                </button>
              ) : (
                <button
                  type="button"
                  className="button secondary compact"
                  onClick={() => inputs.current[kind]?.click()}
                >
                  Browse
                </button>
              )}
              <input
                ref={(el) => {
                  inputs.current[kind] = el;
                }}
                type="file"
                tabIndex={-1}
                className="sr-only"
                accept=".png,image/png"
                aria-label={`${kind} PNG file`}
                onChange={(e) => choose(kind, e.target.files?.[0])}
              />
            </div>
          ))}
        </div>
        <div className="form-two">
          <label>
            Recipe
            <select value={recipe} onChange={(e) => setRecipe(e.target.value as Recipe)}>
              {RECIPES.map((r) => (
                <option key={r} value={r}>
                  {r[0].toUpperCase() + r.slice(1)}
                </option>
              ))}
            </select>
          </label>
          <label>
            Category
            <select value={category} onChange={(e) => setCategory(e.target.value as Category)}>
              {CATEGORIES.map((c) => (
                <option key={c}>{c}</option>
              ))}
            </select>
          </label>
        </div>
        <label className="normal-convention">
          Normal map convention
          <select value={normalY} onChange={(e) => setNormalY(e.target.value as 'directx' | 'opengl')}>
            <option value="directx">DirectX (Y−) — LabPBR default</option>
            <option value="opengl">OpenGL (Y+) — flip green channel</option>
          </select>
        </label>
        <p className="form-hint">
          <ShieldCheck size={15} /> Images are stored on this server, never sent to a third party.
        </p>
        {error && (
          <div className="inline-error" role="alert">
            {error}
          </div>
        )}
        <div className="modal-actions">
          <button type="button" className="button secondary" onClick={onClose} disabled={busy}>
            Cancel
          </button>
          <button className="button primary" disabled={busy}>
            {busy ? <LoaderCircle size={16} className="spin" /> : <ArrowRight size={16} />}
            {busy ? 'Importing maps…' : 'Import material'}
          </button>
        </div>
      </form>
    </Modal>
  );
}
