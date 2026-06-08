import * as fs from 'node:fs';
import * as http from 'node:http';
import * as path from 'node:path';

const MIME_TYPES: Readonly<Record<string, string>> = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.mjs': 'application/javascript',
  '.json': 'application/json',
  '.png': 'image/png',
  '.wasm': 'application/wasm',
};

export interface StaticServerHandle {
  readonly port: number;
  close(): Promise<void>;
}

function createServer(rootDir: string): http.Server {
  const resolvedRoot = path.resolve(rootDir);

  return http.createServer((req: http.IncomingMessage, res: http.ServerResponse) => {
    const urlPath = (req.url ?? '/').split('?')[0] ?? '/';
    const requestedPath = urlPath === '/' ? '/index.html' : urlPath;
    const filePath = path.resolve(resolvedRoot, `.${requestedPath}`);

    if (!filePath.startsWith(resolvedRoot)) {
      res.writeHead(403, { 'Content-Type': 'text/plain' });
      res.end('Forbidden');
      return;
    }

    fs.readFile(filePath, (error: NodeJS.ErrnoException | null, data: Buffer) => {
      if (error !== null) {
        res.writeHead(404, { 'Content-Type': 'text/plain' });
        res.end(`Not found: ${urlPath}`);
        return;
      }

      res.writeHead(200, {
        'Cache-Control': 'no-cache',
        'Content-Type': MIME_TYPES[path.extname(filePath)] ?? 'application/octet-stream',
        'Cross-Origin-Embedder-Policy': 'require-corp',
        'Cross-Origin-Opener-Policy': 'same-origin',
      });
      res.end(data);
    });
  });
}

function closeServer(server: http.Server): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    server.close((error: Error | undefined) => {
      if (error !== undefined) {
        reject(error);
        return;
      }
      resolve();
    });
  });
}

function reservePort(server: http.Server, host: string, initialPort: number, maxPort: number): Promise<number> {
  return new Promise<number>((resolve, reject) => {
    const tryListen = (candidatePort: number): void => {
      const probe = http.createServer();

      probe.once('error', (error: NodeJS.ErrnoException) => {
        if ((error.code === 'EADDRINUSE' || error.code === 'EACCES') && candidatePort < maxPort) {
          tryListen(candidatePort + 1);
          return;
        }
        reject(error);
      });

      probe.once('listening', () => {
        probe.close((probeError: Error | undefined) => {
          if (probeError !== undefined) {
            reject(probeError);
            return;
          }

          server.listen(candidatePort, host, () => {
            resolve(candidatePort);
          });
        });
      });

      probe.listen(candidatePort, host);
    };

    tryListen(initialPort);
  });
}

export async function startStaticServer(
  rootDir: string,
  initialPort: number,
  maxPort = 11_500,
): Promise<StaticServerHandle> {
  const server = createServer(rootDir);
  const host = '127.0.0.1';
  const port = await reservePort(server, host, initialPort, maxPort);

  return {
    port,
    close: () => closeServer(server),
  };
}
