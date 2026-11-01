import {
  ArrowRight,
  Box,
  Check,
  Cpu,
  FileArchive,
  Info,
  Layers3,
  LoaderCircle,
  Settings2,
} from 'lucide-react';
import Modal from './Modal';
import type { MaterialEntry, Settings } from '../../shared/catalog';
export default function BuildDialog({
  entries,
  settings,
  customCount,
  onClose,
  onBuild,
  onSettings,
  busy,
  error,
}: {
  entries: MaterialEntry[];
  settings: Settings;
  customCount: number;
  onClose: () => void;
  onBuild: () => void;
  onSettings: () => void;
  busy: boolean;
  error: string;
}) {
  const maps = 1 + Number(settings.includeNormal) + Number(settings.includeSpecular);
  return (
    <Modal
      title="Ready for a more detailed world?"
      subtitle="Review your selection, then let the CPU engine do its thing."
      onClose={() => {
        if (!busy) onClose();
      }}
    >
      <div className="build-review">
        <div className="build-pack">
          <div className="pack-icon">
            <Box size={31} />
          </div>
          <div>
            <h3>{settings.packName}</h3>
            <p>
              Minecraft Java 26.3 <span>·</span> LabPBR 1.3
            </p>
          </div>
          <span className="badge green">.ZIP</span>
        </div>
        <div className="build-summary">
          <div>
            <Layers3 size={18} />
            <strong>{entries.length}</strong>
            <span>materials</span>
          </div>
          <div>
            <Box size={18} />
            <strong>{settings.resolution}²</strong>
            <span>resolution</span>
          </div>
          <div>
            <FileArchive size={18} />
            <strong>{entries.length * maps}</strong>
            <span>texture maps</span>
          </div>
        </div>
        <div className="review-checks">
          <p>
            <Check size={16} />
            Albedo textures{settings.includeNormal ? ', normal & height maps' : ''}
            {settings.includeSpecular ? ', specular maps' : ''}
          </p>
          <p>
            <Check size={16} />
            Pack metadata and LabPBR properties
          </p>
          <p>
            <Check size={16} />
            Contact sheet and build report included separately
          </p>
          <p>
            <Cpu size={16} />
            CPU generation · {settings.threads} worker threads · seed {settings.seed}
          </p>
        </div>
        <button className="text-button" onClick={onSettings} disabled={busy}>
          <Settings2 size={15} />
          Change pack settings
          <ArrowRight size={14} />
        </button>
        <div className="notice">
          <Info size={18} />
          <p>
            {customCount
              ? `${customCount} imported material${customCount === 1 ? '' : 's'} and ${entries.length - customCount} synthetic demo material${entries.length - customCount === 1 ? '' : 's'}.`
              : 'This selection contains original synthetic demo materials, not scanned assets.'}{' '}
            Use a compatible shader and test the pack in your game before releasing it.
          </p>
        </div>
        {error && (
          <div className="inline-error" role="alert">
            {error}
          </div>
        )}
        <div className="modal-actions">
          <button className="button secondary" onClick={onClose} disabled={busy}>
            Keep editing
          </button>
          <button className="button primary" onClick={onBuild} disabled={busy || !entries.length}>
            {busy ? <LoaderCircle className="spin" size={17} /> : <Box size={17} />}
            {busy ? 'Starting build…' : `Build resource pack`}
          </button>
        </div>
      </div>
    </Modal>
  );
}
