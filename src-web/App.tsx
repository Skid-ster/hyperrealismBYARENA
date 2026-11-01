import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  ArrowDownToLine,
  ArrowLeft,
  ArrowRight,
  ArrowUpRight,
  BookOpen,
  Box,
  Check,
  CheckCheck,
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Clock3,
  Cpu,
  Download,
  ExternalLink,
  FileArchive,
  FolderOpen,
  Grid2X2,
  Heart,
  History,
  ImagePlus,
  Info,
  Layers3,
  Leaf,
  List,
  LoaderCircle,
  Menu,
  MoreHorizontal,
  PackageCheck,
  Plus,
  Search,
  Settings2,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Terminal,
  Trash2,
  TriangleAlert,
  X,
} from 'lucide-react';
import {
  CATALOG,
  CATEGORIES,
  DEFAULT_SETTINGS,
  prettyName,
  resolveEntry,
  toTexture,
  type MaterialEntry,
  type Project,
  type Settings,
  type Texture,
} from '../shared/catalog';
import { api, post, formatBytes, formatDate, type BuildJob, type Health } from './api';
import Modal from './components/Modal';
import Inspector, { staticMaps } from './components/Inspector';
import ImportDialog from './components/ImportDialog';
import SettingsDialog from './components/SettingsDialog';
import BuildDialog from './components/BuildDialog';

type Page = 'library' | 'builds' | 'history' | 'guide';
const INITIAL: Project = {
  edits: {},
  custom: [],
  selected: CATALOG.map((t) => t.name),
  favorites: [],
  settings: DEFAULT_SETTINGS,
};
const pageNames: Record<Page, string> = {
  library: 'Material library',
  builds: 'Build queue',
  history: 'Export history',
  guide: 'Getting started',
};
function Logo({ small = false }: { small?: boolean }) {
  return (
    <div className={`brand ${small ? 'small' : ''}`}>
      <img src="/favicon.svg" alt="" />
      <div>
        <strong>
          HyperRealism<span>™</span>
        </strong>
        <small>RESOURCE PACK STUDIO</small>
      </div>
    </div>
  );
}
function Empty({
  icon,
  title,
  text,
  action,
}: {
  icon: React.ReactNode;
  title: string;
  text: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="empty-state">
      <div>{icon}</div>
      <h2>{title}</h2>
      <p>{text}</p>
      {action}
    </div>
  );
}
export default function App() {
  const [project, setProject] = useState<Project>(INITIAL);
  const projectRef = useRef(project);
  projectRef.current = project;
  const [ready, setReady] = useState(false);
  const [loadError, setLoadError] = useState('');
  const [saveState, setSaveState] = useState<'saved' | 'saving' | 'error'>('saved');
  const saveQueue = useRef<Promise<unknown>>(Promise.resolve());
  const saveTimer = useRef<number | undefined>(undefined);
  const [health, setHealth] = useState<Health | null>(null);
  const [page, setPage] = useState<Page>('library');
  const [mobileNav, setMobileNav] = useState(false);
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('All materials');
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [view, setView] = useState<'grid' | 'list'>('grid');
  const [sort, setSort] = useState('collection');
  const [inspected, setInspected] = useState<string | null>(() => (window.innerWidth > 900 ? 'stone' : null));
  const [dialog, setDialog] = useState<'import' | 'settings' | 'build' | 'about' | null>(null);
  const [jobs, setJobs] = useState<BuildJob[]>([]);
  const [buildBusy, setBuildBusy] = useState(false);
  const [buildError, setBuildError] = useState('');
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [expandedLog, setExpandedLog] = useState<string | null>(null);
  const [toast, setToast] = useState<{ text: string; error: boolean } | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const libraryRef = useRef<HTMLDivElement>(null);
  const notify = useCallback((text: string, error = false) => setToast({ text, error }), []);
  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 5000);
    return () => clearTimeout(t);
  }, [toast]);
  const load = useCallback(async () => {
    setLoadError('');
    try {
      const [p, h, b] = await Promise.all([
        api<Project>('/project'),
        api<Health>('/health'),
        api<BuildJob[]>('/builds'),
      ]);
      setProject(p);
      projectRef.current = p;
      setHealth(h);
      setJobs(b);
      setReady(true);
    } catch (e) {
      setLoadError((e as Error).message);
    }
  }, []);
  useEffect(() => {
    void load();
    return () => clearTimeout(saveTimer.current);
  }, [load]);
  const persist = useCallback(
    (next: Project) => {
      setProject(next);
      projectRef.current = next;
      setSaveState('saving');
      clearTimeout(saveTimer.current);
      saveTimer.current = window.setTimeout(() => {
        saveQueue.current = saveQueue.current
          .catch(() => {})
          .then(() => api('/project', { method: 'PUT', body: JSON.stringify(next) }))
          .then(() => setSaveState('saved'))
          .catch((e) => {
            setSaveState('error');
            notify((e as Error).message, true);
          });
      }, 300);
    },
    [notify],
  );
  const update = useCallback((fn: (p: Project) => Project) => persist(fn(projectRef.current)), [persist]);
  const refreshJobs = useCallback(async () => {
    try {
      setJobs(await api<BuildJob[]>('/builds'));
    } catch {
      /* retain the last visible state */
    }
  }, []);
  const running = jobs.some((j) => j.status === 'running' || j.status === 'queued');
  useEffect(() => {
    if (!running) return;
    const timer = setInterval(refreshJobs, 800);
    return () => clearInterval(timer);
  }, [running, refreshJobs]);
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !dialog) {
        setInspected(null);
        setMobileNav(false);
      }
      if ((e.metaKey || e.ctrlKey) && e.key === 'k' && !dialog) {
        e.preventDefault();
        setPage('library');
        setTimeout(() => searchRef.current?.focus(), 20);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [dialog]);
  const entries = useMemo(
    () => [...CATALOG, ...project.custom].map((t) => resolveEntry(t, project.edits)),
    [project.custom, project.edits],
  );
  const selectedEntries = useMemo(
    () => entries.filter((t) => project.selected.includes(t.name)),
    [entries, project.selected],
  );
  const activeEntry = entries.find((t) => t.name === inspected);
  const filtered = useMemo(() => {
    let result = entries.filter(
      (t) =>
        (category === 'All materials' || t.category === category) &&
        (!favoritesOnly || project.favorites.includes(t.name)) &&
        `${t.label} ${t.name} ${t.category} ${t.recipe}`.toLowerCase().includes(search.toLowerCase()),
    );
    if (sort === 'name') result = [...result].sort((a, b) => a.label.localeCompare(b.label));
    if (sort === 'category') result = [...result].sort((a, b) => a.category.localeCompare(b.category));
    return result;
  }, [entries, category, favoritesOnly, project.favorites, search, sort]);
  const navigate = (next: Page) => {
    setPage(next);
    setMobileNav(false);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };
  const toggleSelected = (name: string) =>
    update((p) => ({
      ...p,
      selected: p.selected.includes(name) ? p.selected.filter((n) => n !== name) : [...p.selected, name],
    }));
  const toggleFavorite = (name: string) =>
    update((p) => ({
      ...p,
      favorites: p.favorites.includes(name) ? p.favorites.filter((n) => n !== name) : [...p.favorites, name],
    }));
  const saveSettings = (settings: Settings) => {
    update((p) => ({ ...p, settings }));
    setDialog(null);
    notify('Pack settings saved.');
  };
  const openBuild = () => {
    setBuildError('');
    if (!project.selected.length) {
      notify('Select at least one material to build your pack.', true);
      return;
    }
    setDialog('build');
  };
  const startBuild = async () => {
    setBuildBusy(true);
    setBuildError('');
    try {
      clearTimeout(saveTimer.current);
      saveQueue.current = saveQueue.current
        .catch(() => {})
        .then(() => api('/project', { method: 'PUT', body: JSON.stringify(projectRef.current) }));
      await saveQueue.current;
      const job = await post<BuildJob>('/builds', {
        names: projectRef.current.selected,
        settings: projectRef.current.settings,
      });
      setJobs((j) => [job, ...j]);
      setDialog(null);
      navigate('builds');
      notify('Your build is queued. You can keep exploring while it runs.');
    } catch (e) {
      setBuildError((e as Error).message);
      setSaveState('error');
    } finally {
      setBuildBusy(false);
    }
  };
  const importMaterial = async (entry: MaterialEntry) => {
    const next = {
      ...projectRef.current,
      custom: [...projectRef.current.custom, entry],
      selected: [...projectRef.current.selected, entry.name],
    };
    try {
      clearTimeout(saveTimer.current);
      saveQueue.current = saveQueue.current
        .catch(() => {})
        .then(() => api('/project', { method: 'PUT', body: JSON.stringify(next) }));
      await saveQueue.current;
      setProject(next);
      projectRef.current = next;
      setSaveState('saved');
      setDialog(null);
      setCategory('All materials');
      setSearch('');
      setFavoritesOnly(false);
      setInspected(entry.name);
      navigate('library');
      notify(`${entry.label} imported and added to your pack.`);
    } catch (e) {
      notify((e as Error).message, true);
      throw e;
    }
  };
  const cancelBuild = async (id: string) => {
    try {
      await post(`/builds/${id}/cancel`, {});
      await refreshJobs();
      notify('Build cancelled.');
    } catch (e) {
      notify((e as Error).message, true);
    }
  };
  const completed = jobs.filter((j) => j.status === 'complete');
  const activeJobs = jobs.filter((j) => j.status === 'running' || j.status === 'queued');
  const visibleSelected = filtered.length > 0 && filtered.every((t) => project.selected.includes(t.name));
  const selectVisible = () =>
    update((p) => ({
      ...p,
      selected: visibleSelected
        ? p.selected.filter((n) => !filtered.some((t) => t.name === n))
        : [...new Set([...p.selected, ...filtered.map((t) => t.name)])],
    }));
  return (
    <div className="app-shell">
      {mobileNav && (
        <button className="sidebar-scrim" aria-label="Close navigation" onClick={() => setMobileNav(false)} />
      )}
      <aside className={`sidebar ${mobileNav ? 'mobile-open' : ''}`}>
        <a
          className="brand-link"
          href="#library"
          onClick={(e) => {
            e.preventDefault();
            navigate('library');
          }}
          aria-label="HyperRealism home"
        >
          <Logo />
        </a>
        <div className="workspace-switch">
          <div className="workspace-avatar">
            <Box size={18} />
          </div>
          <div>
            <strong>My workspace</strong>
            <span>Local project</span>
          </div>
          <span className="workspace-local">LOCAL</span>
        </div>
        <div className="nav-label">WORKSPACE</div>
        <nav aria-label="Main navigation">
          <button className={page === 'library' ? 'active' : ''} onClick={() => navigate('library')}>
            <Grid2X2 size={18} />
            <span>Material library</span>
            <span className="nav-count">{entries.length}</span>
          </button>
          <button className={page === 'builds' ? 'active' : ''} onClick={() => navigate('builds')}>
            <Layers3 size={18} />
            <span>Build queue</span>
            {activeJobs.length > 0 && <span className="nav-count accent">{activeJobs.length}</span>}
          </button>
          <button className={page === 'history' ? 'active' : ''} onClick={() => navigate('history')}>
            <History size={18} />
            <span>Export history</span>
            {completed.length > 0 && <span className="nav-count">{completed.length}</span>}
          </button>
        </nav>
        <div className="nav-label second">RESOURCES</div>
        <nav aria-label="Resources">
          <button onClick={() => setDialog('settings')}>
            <Settings2 size={18} />
            <span>Pack settings</span>
          </button>
          <button className={page === 'guide' ? 'active' : ''} onClick={() => navigate('guide')}>
            <BookOpen size={18} />
            <span>Getting started</span>
            <ArrowUpRight size={14} />
          </button>
        </nav>
        <div className="sidebar-bottom">
          <div className="engine-card">
            <div>
              <span className={`status-dot ${health?.engine ? '' : 'warning'}`} />
              <strong>
                {health?.engine ? 'CPU engine ready' : ready ? 'Engine not built' : 'Connecting to engine'}
              </strong>
              <Cpu size={16} />
            </div>
            <p>
              Local processing.
              <br />
              Your textures stay yours.
            </p>
            <div className="engine-card-footer">
              <span>RUST CORE</span>
              <span>v0.1.0</span>
            </div>
          </div>
          <button className="profile" onClick={() => setDialog('about')}>
            <div className="avatar">HR</div>
            <div>
              <strong>Local creator</strong>
              <span>Personal workspace</span>
            </div>
            <MoreHorizontal size={18} />
          </button>
        </div>
      </aside>
      <div className="main-shell">
        <header className="topbar">
          <div className="breadcrumb">
            <button
              className="icon-button mobile-menu"
              aria-label="Open navigation"
              onClick={() => setMobileNav(true)}
            >
              <Menu size={21} />
            </button>
            <span>Workspace</span>
            <ChevronRight size={14} />
            <strong>{pageNames[page]}</strong>
          </div>
          <div className="topbar-right">
            <span
              className={`save-status ${saveState}`}
              title={
                saveState === 'error'
                  ? 'Changes are in memory but not saved. Use Retry to reconnect.'
                  : 'Saved to this server'
              }
            >
              {saveState === 'saving' ? (
                <LoaderCircle className="spin" size={15} />
              ) : saveState === 'error' ? (
                <TriangleAlert size={15} />
              ) : (
                <CheckCheck size={16} />
              )}
              <span>
                {saveState === 'saving'
                  ? 'Saving changes…'
                  : saveState === 'error'
                    ? 'Not saved'
                    : 'All changes saved'}
              </span>
            </span>
            <div className="topbar-divider" />
            <button
              className="icon-button"
              onClick={() => navigate('guide')}
              aria-label="Help and getting started"
            >
              <CircleHelp size={19} />
            </button>
            <div className="topbar-avatar">HR</div>
          </div>
        </header>
        <main>
          {loadError && (
            <div className="connection-banner" role="alert">
              <TriangleAlert size={18} />
              <span>{loadError}</span>
              <button className="button secondary compact" onClick={load}>
                Retry connection
              </button>
            </div>
          )}
          {page === 'library' && (
            <>
              <div className="page-heading">
                <div>
                  <div className="title-with-badge">
                    <h1>Material library</h1>
                    <span className="badge">{entries.length} materials</span>
                  </div>
                  <p>Extraordinary detail. One block at a time.</p>
                </div>
                <div className="heading-actions">
                  <button className="button secondary" onClick={() => setDialog('import')} disabled={!ready}>
                    <Plus size={17} />
                    Import material
                  </button>
                  <button className="button primary" onClick={openBuild} disabled={!ready}>
                    <Box size={17} />
                    Build pack<span className="button-count">{project.selected.length}</span>
                  </button>
                </div>
              </div>
              <section className="hero-banner">
                <img
                  className="hero-art"
                  src="/images/world-hero.jpg"
                  alt="A richly detailed voxel island with mossy stone, oak trees, and a waterfall"
                />
                <div className="hero-shade" />
                <div className="hero-content">
                  <div className="hero-eyebrow">
                    <span /> A NEW LEVEL OF DETAIL
                  </div>
                  <h2>
                    Your world.
                    <br />
                    <span>Reimagined.</span>
                  </h2>
                  <p>
                    Craft a richer Minecraft with physically based
                    <br className="desktop-br" /> materials. Built locally. Made for your world.
                  </p>
                  <button
                    onClick={() => libraryRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' })}
                  >
                    Explore the collection
                    <ArrowRight size={16} />
                  </button>
                </div>
                <div className="hero-caption">
                  <Sparkles size={12} />
                  Illustrative world render
                </div>
                <div className="hero-tech">
                  <span>
                    <Box size={13} />
                    128× DETAIL
                  </span>
                  <i />
                  <span>
                    <Layers3 size={13} />
                    LabPBR 1.3
                  </span>
                  <i />
                  <span>
                    <Cpu size={13} />
                    CPU POWERED
                  </span>
                </div>
              </section>
              <div className={`library-layout ${activeEntry ? 'with-inspector' : ''}`} ref={libraryRef}>
                <section className="library-content" aria-label="Materials">
                  <div className="library-toolbar">
                    <div className="search-field">
                      <Search size={17} />
                      <input
                        ref={searchRef}
                        placeholder="Search materials…"
                        aria-label="Search materials"
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                      />
                      {search ? (
                        <button
                          className="icon-button small"
                          onClick={() => setSearch('')}
                          aria-label="Clear search"
                        >
                          <X size={14} />
                        </button>
                      ) : (
                        <kbd>⌘ K</kbd>
                      )}
                    </div>
                    <div className="view-toggle">
                      <button
                        className={view === 'grid' ? 'active' : ''}
                        onClick={() => setView('grid')}
                        aria-label="Grid view"
                        aria-pressed={view === 'grid'}
                      >
                        <Grid2X2 size={16} />
                      </button>
                      <button
                        className={view === 'list' ? 'active' : ''}
                        onClick={() => setView('list')}
                        aria-label="List view"
                        aria-pressed={view === 'list'}
                      >
                        <List size={17} />
                      </button>
                    </div>
                  </div>
                  <div className="category-tabs" role="group" aria-label="Material categories">
                    {['All materials', ...CATEGORIES].map((c) => (
                      <button
                        key={c}
                        aria-pressed={category === c}
                        className={category === c ? 'active' : ''}
                        onClick={() => setCategory(c)}
                      >
                        {c}
                        {c === 'All materials' && <span>{entries.length}</span>}
                      </button>
                    ))}
                  </div>
                  <div className="results-toolbar">
                    <label className="select-all">
                      <input
                        type="checkbox"
                        checked={visibleSelected}
                        onChange={selectVisible}
                        disabled={!filtered.length}
                      />
                      <span>{filtered.length} materials</span>
                    </label>
                    <div>
                      <button
                        className={`filter-button ${favoritesOnly ? 'active' : ''}`}
                        onClick={() => setFavoritesOnly((v) => !v)}
                        aria-pressed={favoritesOnly}
                      >
                        <Heart size={14} fill={favoritesOnly ? 'currentColor' : 'none'} />
                        <span>Favorites</span>
                      </button>
                      <span className="small-divider" />
                      <select
                        aria-label="Sort materials"
                        value={sort}
                        onChange={(e) => setSort(e.target.value)}
                      >
                        <option value="collection">Collection order</option>
                        <option value="name">Name A–Z</option>
                        <option value="category">Category</option>
                      </select>
                    </div>
                  </div>
                  {!ready && !loadError ? (
                    <div className="material-grid">
                      {Array.from({ length: 6 }, (_, i) => (
                        <div className="skeleton-card" key={i} />
                      ))}
                    </div>
                  ) : filtered.length ? (
                    <div className={`material-grid ${view === 'list' ? 'list-view' : ''}`}>
                      {filtered.map((entry) => {
                        const selected = project.selected.includes(entry.name);
                        const favorite = project.favorites.includes(entry.name);
                        return (
                          <article
                            key={entry.name}
                            className={`material-card ${inspected === entry.name ? 'inspected' : ''} ${selected ? 'selected' : ''}`}
                          >
                            <button
                              className="material-open"
                              onClick={() => setInspected(entry.name)}
                              aria-label={`Inspect ${entry.label}`}
                            >
                              <div
                                className={`material-image checkerboard category-${entry.category.split(' ')[0].toLowerCase()}`}
                              >
                                {entry.custom ? (
                                  <CustomThumbnail entry={entry} seed={project.settings.seed} />
                                ) : (
                                  <img
                                    src={`/previews/${entry.name}.png`}
                                    alt={`${entry.label} albedo texture`}
                                    loading="lazy"
                                  />
                                )}
                                <span className="texture-resolution">128 × 128</span>
                                <span className="material-image-shade" />
                              </div>
                              <div className="material-card-info">
                                <div>
                                  <h3>{entry.label}</h3>
                                  <span className="material-category">{entry.category}</span>
                                </div>
                                <ArrowUpRight size={16} />
                              </div>
                              <div className="material-card-meta">
                                <span className="mono">{entry.name}.png</span>
                                <span className="map-badges">
                                  <i>A</i>
                                  {project.settings.includeNormal && <i>N</i>}
                                  {project.settings.includeSpecular && <i>S</i>}
                                </span>
                              </div>
                            </button>
                            <button
                              className={`card-checkbox ${selected ? 'checked' : ''}`}
                              role="checkbox"
                              aria-checked={selected}
                              aria-label={`${selected ? 'Exclude' : 'Include'} ${entry.label} in pack`}
                              onClick={() => toggleSelected(entry.name)}
                            >
                              {selected && <Check size={12} strokeWidth={3} />}
                            </button>
                            <button
                              className={`card-favorite ${favorite ? 'is-favorite' : ''}`}
                              onClick={() => toggleFavorite(entry.name)}
                              aria-label={`${favorite ? 'Unfavorite' : 'Favorite'} ${entry.label}`}
                              aria-pressed={favorite}
                            >
                              <Heart size={14} fill={favorite ? 'currentColor' : 'none'} />
                            </button>
                            {entry.custom && <span className="custom-badge">IMPORTED</span>}
                          </article>
                        );
                      })}
                    </div>
                  ) : (
                    <Empty
                      icon={<Search size={28} />}
                      title={favoritesOnly ? 'No favorites here yet' : 'No materials found'}
                      text={
                        favoritesOnly
                          ? 'Tap the heart on a material to keep it close.'
                          : 'Try another search or choose a different category.'
                      }
                      action={
                        <button
                          className="button secondary"
                          onClick={() => {
                            setSearch('');
                            setCategory('All materials');
                            setFavoritesOnly(false);
                          }}
                        >
                          Show all materials
                        </button>
                      }
                    />
                  )}
                  <div className="library-footer">
                    <span>
                      <ShieldCheck size={14} />
                      Original demo textures · locally generated
                    </span>
                    <span>
                      {filtered.length} of {entries.length} materials
                    </span>
                  </div>
                </section>
                {activeEntry && (
                  <button
                    className="inspector-scrim"
                    aria-label="Close inspector overlay"
                    onClick={() => setInspected(null)}
                  />
                )}
                {activeEntry && (
                  <Inspector
                    key={activeEntry.name}
                    entry={activeEntry}
                    selected={project.selected.includes(activeEntry.name)}
                    favorite={project.favorites.includes(activeEntry.name)}
                    seed={project.settings.seed}
                    onClose={() => setInspected(null)}
                    onSelect={() => toggleSelected(activeEntry.name)}
                    onFavorite={() => toggleFavorite(activeEntry.name)}
                    onSave={(texture: Partial<Texture>) =>
                      update((p) => ({ ...p, edits: { ...p.edits, [activeEntry.name]: texture } }))
                    }
                    onReset={() =>
                      update((p) => {
                        const edits = { ...p.edits };
                        delete edits[activeEntry.name];
                        return { ...p, edits };
                      })
                    }
                    notify={notify}
                  />
                )}
              </div>
            </>
          )}
          {(page === 'builds' || page === 'history') && (
            <>
              <div className="page-heading">
                <div>
                  <div className="title-with-badge">
                    <h1>{pageNames[page]}</h1>
                    <span className="badge">
                      {page === 'builds' ? activeJobs.length : completed.length}{' '}
                      {page === 'builds' ? 'active' : 'exports'}
                    </span>
                  </div>
                  <p>
                    {page === 'builds'
                      ? 'From a collection of materials to a world of possibilities.'
                      : 'Your finished packs, ready for their next adventure.'}
                  </p>
                </div>
                <button className="button primary" onClick={openBuild}>
                  <Plus size={17} />
                  New build
                </button>
              </div>
              <div className="notice page-notice">
                <Cpu size={19} />
                <p>
                  <strong>Built on your machine.</strong> The Rust CPU engine processes one pack at a time.
                  You can continue editing while a build runs; each build uses the selection saved when it
                  started.
                </p>
              </div>
              <div className="jobs-list">
                {(page === 'builds' ? jobs : completed).map((job) => (
                  <article className={`job-card ${job.status}`} key={job.id}>
                    <div className="job-heading">
                      <div className="job-icon">
                        {job.status === 'complete' ? (
                          <PackageCheck size={25} />
                        ) : job.status === 'failed' ? (
                          <TriangleAlert size={25} />
                        ) : (
                          <Layers3 size={25} />
                        )}
                      </div>
                      <div>
                        <h2>{job.name}</h2>
                        <p>
                          {formatDate(job.createdAt)} <span>·</span> {job.total} materials <span>·</span>{' '}
                          {job.settings.resolution} × {job.settings.resolution}
                        </p>
                      </div>
                      <span className={`badge job-status ${job.status}`}>
                        {job.status === 'running' && <span className="status-dot" />}
                        {job.status === 'complete' ? 'Ready to download' : job.status}
                      </span>
                    </div>
                    {['running', 'queued'].includes(job.status) && (
                      <div className="job-progress">
                        <div>
                          <span>
                            {job.current === 'Waiting for the CPU engine'
                              ? job.current
                              : `Generating ${prettyName(job.current)}`}
                          </span>
                          <strong>
                            {job.completed} / {job.total}
                          </strong>
                        </div>
                        <div className="progress-track">
                          <span style={{ width: `${Math.max(2, (job.completed / job.total) * 100)}%` }} />
                        </div>
                      </div>
                    )}
                    {job.error && <div className="inline-error">{job.error}</div>}
                    {job.status === 'complete' && (
                      <div className="job-output">
                        <FileArchive size={20} />
                        <div>
                          <strong>
                            {job.name}-{job.settings.resolution}.zip
                          </strong>
                          <span>
                            {formatBytes(job.bytes || 0)} ·{' '}
                            {job.total *
                              (1 +
                                Number(job.settings.includeNormal) +
                                Number(job.settings.includeSpecular))}{' '}
                            maps · ZIP verified by engine
                          </span>
                        </div>
                        <CheckCheck size={19} />
                      </div>
                    )}
                    <div className="job-actions">
                      <div>
                        <button
                          className="text-button muted"
                          onClick={() => setExpandedLog(expandedLog === job.id ? null : job.id)}
                          aria-expanded={expandedLog === job.id}
                        >
                          <Terminal size={15} />
                          {expandedLog === job.id ? 'Hide build log' : 'View build log'}
                        </button>
                        {job.status === 'complete' && (
                          <>
                            <a
                              className="text-button muted"
                              href={`/api/builds/${job.id}/sheet`}
                              target="_blank"
                              rel="noreferrer"
                            >
                              <Grid2X2 size={14} />
                              Contact sheet
                            </a>
                            <a
                              className="text-button muted"
                              href={`/api/builds/${job.id}/report`}
                              target="_blank"
                              rel="noreferrer"
                            >
                              Build report
                              <ArrowUpRight size={13} />
                            </a>
                          </>
                        )}
                      </div>
                      <div>
                        {['queued', 'running'].includes(job.status) && (
                          <button className="button secondary compact" onClick={() => cancelBuild(job.id)}>
                            Cancel build
                          </button>
                        )}
                        {job.status === 'complete' && (
                          <>
                            <button
                              className="icon-button"
                              aria-label={`Delete export ${job.name}`}
                              onClick={() => setDeleteId(job.id)}
                            >
                              <Trash2 size={16} />
                            </button>
                            <a className="button primary compact" href={`/api/builds/${job.id}/download`}>
                              <Download size={16} />
                              Download pack
                            </a>
                          </>
                        )}
                        {['failed', 'cancelled'].includes(job.status) && (
                          <button className="button secondary compact" onClick={openBuild}>
                            Start a new build
                            <ArrowRight size={14} />
                          </button>
                        )}
                      </div>
                    </div>
                    {expandedLog === job.id && (
                      <pre className="build-log" aria-label="Build log">
                        {job.logs.join('\n') || 'Waiting for engine output…'}
                      </pre>
                    )}
                  </article>
                ))}
                {!(page === 'builds' ? jobs : completed).length && (
                  <Empty
                    icon={page === 'builds' ? <Layers3 size={34} /> : <FolderOpen size={34} />}
                    title={
                      page === 'builds' ? 'A clear queue. A fresh start.' : 'Your first export is waiting'
                    }
                    text={
                      page === 'builds'
                        ? 'Select your favorite materials and build a pack. Progress will appear here in real time.'
                        : 'Complete a build and your downloadable resource packs will appear here.'
                    }
                    action={
                      <button
                        className="button primary"
                        onClick={() => (page === 'builds' ? openBuild() : navigate('library'))}
                      >
                        {page === 'builds' ? 'Build a resource pack' : 'Explore materials'}
                        <ArrowRight size={16} />
                      </button>
                    }
                  />
                )}
              </div>
            </>
          )}
          {page === 'guide' && (
            <>
              <div className="page-heading">
                <div>
                  <div className="title-with-badge">
                    <h1>A better world starts here.</h1>
                  </div>
                  <p>A practical guide from your first material to your first export.</p>
                </div>
                <button className="button secondary" onClick={() => navigate('library')}>
                  <ArrowLeft size={16} />
                  Back to library
                </button>
              </div>
              <div className="guide-hero">
                <Logo />
                <span className="badge green">STUDIO GUIDE</span>
                <h2>
                  Make it detailed.
                  <br />
                  Make it yours.
                </h2>
                <p>
                  HyperRealism is a local-first texture workshop, powered by a real Rust CPU pipeline. No
                  accounts. No cloud uploads. Just your materials.
                </p>
              </div>
              <div className="guide-grid">
                <section className="guide-card">
                  <span className="step-number">01</span>
                  <h2>Curate your collection</h2>
                  <p>
                    Explore the 24 original synthetic demos, or import your own albedo PNG. Normal and
                    roughness maps are optional. Select the materials you want using the checkbox on each
                    card.
                  </p>
                  <button className="text-button" onClick={() => navigate('library')}>
                    Open material library
                    <ArrowRight size={15} />
                  </button>
                </section>
                <section className="guide-card">
                  <span className="step-number">02</span>
                  <h2>Dial in the surface</h2>
                  <p>
                    Click a material to inspect its 3D block and map channels. Adjust tint, roughness, and
                    normal strength, then choose <strong>Apply & preview</strong>. Your edits are saved to
                    this workspace.
                  </p>
                  <p>The 3D view is illustrative, not a Minecraft shader implementation.</p>
                </section>
                <section className="guide-card">
                  <span className="step-number">03</span>
                  <h2>Build & export</h2>
                  <p>
                    Choose your resolution and PBR channels in Pack settings. Build the pack, follow real
                    engine progress, and download the ZIP. A contact sheet and build report are available
                    separately.
                  </p>
                  <button className="text-button" onClick={() => setDialog('settings')}>
                    Review pack settings
                    <ArrowRight size={15} />
                  </button>
                </section>
                <section className="guide-card">
                  <span className="step-number">04</span>
                  <h2>Take it into your world</h2>
                  <p>
                    Place the downloaded ZIP into your Minecraft <code>resourcepacks</code> folder, then
                    enable it in Options → Resource Packs. Put HyperRealism above your base pack.
                  </p>
                  <p>
                    Use a LabPBR 1.3-compatible shader to see normal and specular effects. Without one, only
                    the base colors are used.
                  </p>
                </section>
              </div>
              <section className="guide-notes">
                <h2>
                  <Info size={20} />A note on scope & compatibility
                </h2>
                <p>
                  This studio includes <strong>24 synthetic demonstration materials</strong>, not a
                  280-texture production catalog or licensed Poly Haven scans. The banner is an illustrative
                  render. Names target Minecraft Java block paths; they have not been audited against a 26.3
                  game JAR. Animations, items, entities, and GUI textures are outside this collection.
                </p>
                <p>
                  The pack uses the project’s target metadata (<code>min_format: [97, 1]</code>,{' '}
                  <code>max_format: 97</code>). In-game and shader validation remain necessary. CPU generation
                  works independently of a GPU; no RTX, Vulkan, FPS, or hardware-speed claims are made.
                </p>
                <div>
                  <a href="https://github.com/Skid-ster/hyperrealismBYARENA" target="_blank" rel="noreferrer">
                    Project repository
                    <ExternalLink size={14} />
                  </a>
                  <a
                    href="https://shaders.properties/current/how-to/pbr_standards/"
                    target="_blank"
                    rel="noreferrer"
                  >
                    LabPBR reference
                    <ExternalLink size={14} />
                  </a>
                </div>
              </section>
            </>
          )}
        </main>
        <footer className="app-footer">
          <span>
            HYPERREALISM STUDIO <i /> BUILT FOR THE BLOCKS.
          </span>
          <button onClick={() => setDialog('about')}>
            Local-first. Creator-owned.
            <ArrowUpRight size={12} />
          </button>
        </footer>
      </div>
      {dialog === 'import' && (
        <ImportDialog
          onClose={() => setDialog(null)}
          onImport={importMaterial}
          existingNames={entries.map((t) => t.name)}
        />
      )}
      {dialog === 'settings' && (
        <SettingsDialog settings={project.settings} onSave={saveSettings} onClose={() => setDialog(null)} />
      )}
      {dialog === 'build' && (
        <BuildDialog
          entries={selectedEntries}
          settings={project.settings}
          customCount={selectedEntries.filter((t) => t.custom).length}
          onClose={() => setDialog(null)}
          onBuild={startBuild}
          onSettings={() => setDialog('settings')}
          busy={buildBusy}
          error={buildError}
        />
      )}
      {dialog === 'about' && (
        <Modal
          title="Your own corner of the overworld."
          subtitle="HyperRealism Resource Pack Studio"
          onClose={() => setDialog(null)}
        >
          <div className="about-content">
            <Logo />
            <p>
              A local-first workspace for creating Minecraft block textures and LabPBR maps with the Rust CPU
              engine.
            </p>
            <div className="notice">
              <ShieldCheck size={20} />
              <p>
                Your project and imported PNGs are saved in this server’s <code>.workspace</code> directory.
                Anyone with access to this server can access the workspace; this is not a multi-user
                authenticated service.
              </p>
            </div>
            <p>CPU engine: {health?.engine ? 'available' : 'not built'} · LabPBR 1.3 · Studio 1.0.0</p>
            <button className="button primary" onClick={() => setDialog(null)}>
              Back to creating
              <ArrowRight size={16} />
            </button>
          </div>
        </Modal>
      )}
      {deleteId && (
        <Modal
          title="Delete this export?"
          subtitle="This removes the ZIP, contact sheet, and build report from the server."
          onClose={() => setDeleteId(null)}
        >
          <div className="modal-actions padded">
            <button className="button secondary" onClick={() => setDeleteId(null)}>
              Keep export
            </button>
            <button
              className="button danger"
              onClick={async () => {
                try {
                  await api(`/builds/${deleteId}`, { method: 'DELETE' });
                  await refreshJobs();
                  setDeleteId(null);
                  notify('Export deleted.');
                } catch (e) {
                  notify((e as Error).message, true);
                }
              }}
            >
              <Trash2 size={16} />
              Delete export
            </button>
          </div>
        </Modal>
      )}
      {toast && (
        <div className={`toast ${toast.error ? 'error' : ''}`} role={toast.error ? 'alert' : 'status'}>
          {toast.error ? <TriangleAlert size={19} /> : <Check size={19} />}
          <span>{toast.text}</span>
          <button
            className="icon-button small"
            aria-label="Dismiss notification"
            onClick={() => setToast(null)}
          >
            <X size={16} />
          </button>
        </div>
      )}
    </div>
  );
}
function CustomThumbnail({ entry, seed }: { entry: MaterialEntry; seed: number }) {
  const [url, setUrl] = useState('');
  useEffect(() => {
    let alive = true;
    post<{ albedo: string }>('/previews', {
      texture: toTexture(entry),
      seed,
      assets: entry.assets,
      normal_y: entry.normal_y || 'directx',
    })
      .then((r) => {
        if (alive) setUrl(r.albedo);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [entry.name, seed]);
  return url ? (
    <img src={url} alt={`${entry.label} albedo texture`} />
  ) : (
    <div className="custom-thumb-loading">
      <ImagePlus size={28} />
      <span>Preparing preview</span>
    </div>
  );
}
