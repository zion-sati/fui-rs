export const ASSET_FETCH_ATTEMPTS = 4;
const ASSET_RETRY_DELAY_MS = 100;
const scriptSourceCache = new Map<string, Promise<string>>();

function delay(ms: number): Promise<void> {
  return new Promise<void>((resolve) => {
    window.setTimeout(resolve, ms);
  });
}

export function resolveAssetUrl(url: string): string {
  return new URL(url, document.baseURI).toString();
}

export function normalizeFetchIntegrity(integrity: string | null | undefined): string | null {
  if (integrity === null || integrity === undefined || integrity.length === 0) {
    return null;
  }
  const value = integrity;
  if (!value.startsWith('sha256-')) {
    return value;
  }
  let digest = value.slice(7).replace(/-/g, '+').replace(/_/g, '/');
  while ((digest.length % 4) !== 0) {
    digest += '=';
  }
  return `sha256-${digest}`;
}

export function buildFetchInit(integrity: string | null | undefined, cache: RequestCache = 'force-cache'): RequestInit {
  const fetchIntegrity = normalizeFetchIntegrity(integrity);
  const init: RequestInit = {
    credentials: 'same-origin',
    cache,
  };
  if (fetchIntegrity !== null) {
    init.integrity = fetchIntegrity;
  }
  return init;
}

export async function fetchWithRetry<T>(
  url: string,
  attempts: number,
  read: (response: Response) => T | Promise<T>,
  init?: RequestInit,
): Promise<T> {
  let lastError: unknown = null;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      const response = await fetch(url, init);
      if (!response.ok) {
        throw new Error(`Failed to fetch ${url}: ${String(response.status)}`);
      }
      return await read(response);
    } catch (error: unknown) {
      lastError = error;
      if (attempt === attempts) {
        break;
      }
      await delay(ASSET_RETRY_DELAY_MS * attempt);
    }
  }
  throw lastError instanceof Error ? lastError : new Error(`Failed to fetch ${url}`);
}

export async function fetchBinaryAsset(url: string, integrity: string | null | undefined): Promise<Uint8Array> {
  const assetUrl = resolveAssetUrl(url);
  const buffer = await fetchWithRetry<ArrayBuffer>(
    assetUrl,
    ASSET_FETCH_ATTEMPTS,
    async (response) => await response.arrayBuffer(),
    buildFetchInit(integrity),
  );
  return new Uint8Array(buffer);
}

export async function fetchResponseWithRetry(url: string, integrity: string | null | undefined): Promise<Response> {
  const assetUrl = resolveAssetUrl(url);
  return await fetchWithRetry<Response>(
    assetUrl,
    ASSET_FETCH_ATTEMPTS,
    (response) => response,
    buildFetchInit(integrity),
  );
}

export async function loadScriptResource(scriptUrl: string, integrity: string | null | undefined): Promise<void> {
  const absoluteUrl = resolveAssetUrl(scriptUrl);
  const sourceText = await fetchScriptSource(absoluteUrl, integrity);
  const blobSource = sourceText.includes('//# sourceURL=')
    ? sourceText
    : `${sourceText}\n//# sourceURL=${absoluteUrl.replace(/\s/g, '%20')}`;
  const blobUrl = URL.createObjectURL(new Blob([blobSource], { type: 'application/javascript' }));
  try {
    await new Promise<void>((resolve, reject) => {
      const script = document.createElement('script');
      script.src = blobUrl;
      script.async = true;
      script.addEventListener('load', () => {
        script.remove();
        resolve();
      });
      script.addEventListener('error', () => {
        script.remove();
        reject(new Error(`Failed to execute ${absoluteUrl}`));
      });
      document.head.appendChild(script);
    });
  } finally {
    URL.revokeObjectURL(blobUrl);
  }
}

export async function fetchScriptSource(scriptUrl: string, integrity: string | null | undefined): Promise<string> {
  const absoluteUrl = resolveAssetUrl(scriptUrl);
  const cacheKey = `${absoluteUrl}::${integrity ?? ''}`;
  let sourcePromise = scriptSourceCache.get(cacheKey);
  if (sourcePromise === undefined) {
    sourcePromise = fetchBinaryAsset(absoluteUrl, integrity).then((scriptBytes) =>
      new TextDecoder('utf-8').decode(scriptBytes));
    scriptSourceCache.set(cacheKey, sourcePromise);
  }
  return await sourcePromise;
}
