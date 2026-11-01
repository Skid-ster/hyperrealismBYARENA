export async function api<T>(path: string, options: RequestInit = {}): Promise<T> {
  let response: Response;
  try {
    const headers = new Headers(options.headers);
    if (options.body && !(options.body instanceof FormData) && !headers.has('Content-Type'))
      headers.set('Content-Type', 'application/json');
    response = await fetch(`/api${path}`, { ...options, headers });
  } catch {
    throw new Error('Cannot reach the local server. Check your connection and try again.');
  }
  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error(body.error || `Request failed (${response.status}). Please try again.`);
  }
  if (response.status === 204) return undefined as T;
  return response.json();
}
export const post = <T>(path: string, body: unknown) =>
  api<T>(path, { method: 'POST', body: JSON.stringify(body) });
export interface MapUrls {
  albedo: string;
  normal: string;
  specular: string;
}
export interface Health {
  status: string;
  engine: boolean;
  version: string;
  backend: string;
  labpbr: string;
  target: string;
  demoTextures: number;
}
export interface BuildJob {
  id: string;
  name: string;
  status: 'queued' | 'running' | 'complete' | 'failed' | 'cancelled';
  createdAt: string;
  completedAt?: string;
  total: number;
  completed: number;
  current: string;
  error?: string;
  bytes?: number;
  logs: string[];
  settings: {
    resolution: number;
    seed: number;
    threads: number;
    includeNormal: boolean;
    includeSpecular: boolean;
    packName: string;
  };
}
export const formatBytes = (n: number) =>
  n < 1024 * 1024 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1024 / 1024).toFixed(1)} MB`;
export const formatDate = (date: string) =>
  new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(date));
