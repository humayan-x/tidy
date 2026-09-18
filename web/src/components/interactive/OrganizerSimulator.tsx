import { createSignal, For, Show } from 'solid-js';
import iconTerminal from '@tabler/icons/outline/terminal-2.svg?raw';
import iconHistory from '@tabler/icons/outline/history.svg?raw';
import iconFile from '@tabler/icons/outline/file.svg?raw';
import iconAlertTriangle from '@tabler/icons/outline/alert-triangle.svg?raw';
import iconCheck from '@tabler/icons/outline/check.svg?raw';

interface FileItem {
  id: string;
  name: string;
  originalPath: string;
  organizedPath: string;
  category: string;
  ext: string;
  size: string;
  isDownloadGuard?: boolean;
  isCollision?: boolean;
  status: 'normal' | 'guard' | 'collision';
}

const INITIAL_FILES: FileItem[] = [
  {
    id: '1',
    name: 'invoice_2026_q1.pdf',
    originalPath: '~/Downloads/invoice_2026_q1.pdf',
    organizedPath: '~/Downloads/Documents/PDF/invoice_2026_q1.pdf',
    category: 'Documents',
    ext: 'pdf',
    size: '1.4 MB',
    status: 'normal',
  },
  {
    id: '2',
    name: 'quarterly_report.xlsx',
    originalPath: '~/Downloads/quarterly_report.xlsx',
    organizedPath: '~/Downloads/Documents/XLSX/quarterly_report.xlsx',
    category: 'Documents',
    ext: 'xlsx',
    size: '840 KB',
    status: 'normal',
  },
  {
    id: '3',
    name: 'screenshot_dashboard.png',
    originalPath: '~/Downloads/screenshot_dashboard.png',
    organizedPath: '~/Downloads/Images/PNG/screenshot_dashboard.png',
    category: 'Images',
    ext: 'png',
    size: '2.1 MB',
    status: 'normal',
  },
  {
    id: '4',
    name: 'screenshot_dashboard.png',
    originalPath: '~/Downloads/screenshot_dashboard.png',
    organizedPath: '~/Downloads/Images/PNG/screenshot_dashboard (1).png',
    category: 'Images',
    ext: 'png',
    size: '1.8 MB',
    isCollision: true,
    status: 'collision',
  },
  {
    id: '5',
    name: 'release_v2.0.tar.gz',
    originalPath: '~/Downloads/release_v2.0.tar.gz',
    organizedPath: '~/Downloads/Archives/TAR_GZ/release_v2.0.tar.gz',
    category: 'Archives',
    ext: 'tar.gz',
    size: '42.6 MB',
    status: 'normal',
  },
  {
    id: '6',
    name: 'large_dataset.zip.crdownload',
    originalPath: '~/Downloads/large_dataset.zip.crdownload',
    organizedPath: '~/Downloads/large_dataset.zip.crdownload',
    category: 'Ignored',
    ext: 'crdownload',
    size: '1.2 GB (writing)',
    isDownloadGuard: true,
    status: 'guard',
  },
  {
    id: '7',
    name: 'cluster_deploy.rs',
    originalPath: '~/Downloads/cluster_deploy.rs',
    organizedPath: '~/Downloads/Code/RS/cluster_deploy.rs',
    category: 'Code',
    ext: 'rs',
    size: '14 KB',
    status: 'normal',
  },
  {
    id: '8',
    name: 'podcast_episode_42.mp3',
    originalPath: '~/Downloads/podcast_episode_42.mp3',
    organizedPath: '~/Downloads/Audio/MP3/podcast_episode_42.mp3',
    category: 'Audio',
    ext: 'mp3',
    size: '68 MB',
    status: 'normal',
  },
];

export default function OrganizerSimulator() {
  const [isOrganized, setIsOrganized] = createSignal(false);
  const [activeTab, setActiveTab] = createSignal<'all' | 'documents' | 'images' | 'archives' | 'code' | 'audio'>('all');
  const [lastAction, setLastAction] = createSignal<string | null>(null);

  const handleOrganize = () => {
    setIsOrganized(true);
    setLastAction('Ran tidy run: 7 files moved to category folders, 1 collision renamed safely, 1 active download guarded. SQLite batch recorded.');
  };

  const handleUndo = () => {
    setIsOrganized(false);
    setLastAction('Ran tidy undo: Restored 7 files to root ~/Downloads. Empty category folders pruned cleanly.');
  };

  const filteredFiles = () => {
    const tab = activeTab();
    if (tab === 'all') return INITIAL_FILES;
    return INITIAL_FILES.filter((f) => f.category.toLowerCase() === tab);
  };

  return (
    <div class="rounded-lg border border-zinc-800 bg-zinc-900 overflow-hidden shadow-2xl">
      {/* Terminal Titlebar */}
      <div class="flex items-center justify-between border-b border-zinc-800 bg-zinc-950 px-4 py-3">
        <div class="flex items-center gap-2">
          <div class="flex gap-1.5">
            <div class="h-3 w-3 rounded-full bg-zinc-700" />
            <div class="h-3 w-3 rounded-full bg-zinc-700" />
            <div class="h-3 w-3 rounded-full bg-zinc-700" />
          </div>
          <span class="ml-2 font-mono text-xs text-zinc-400">
            tidy-simulator — {isOrganized() ? '~/Downloads (Organized Tree)' : '~/Downloads (Unsorted Flat)'}
          </span>
        </div>

        {/* Action Buttons */}
        <div class="flex items-center gap-2">
          <button
            onClick={handleOrganize}
            disabled={isOrganized()}
            class={`inline-flex items-center gap-1.5 rounded px-3 py-1 font-mono text-xs font-medium transition-colors ${
              isOrganized()
                ? 'border border-zinc-800 bg-zinc-800/50 text-zinc-500 cursor-not-allowed'
                : 'border border-cyan-600 bg-cyan-950/60 text-cyan-300 hover:bg-cyan-900/60'
            }`}
          >
            <span class="w-3.5 h-3.5 flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconTerminal} />
            <span>Run tidy</span>
          </button>

          <button
            onClick={handleUndo}
            disabled={!isOrganized()}
            class={`inline-flex items-center gap-1.5 rounded px-3 py-1 font-mono text-xs font-medium transition-colors ${
              !isOrganized()
                ? 'border border-zinc-800 bg-zinc-800/50 text-zinc-500 cursor-not-allowed'
                : 'border border-zinc-700 bg-zinc-800 text-zinc-200 hover:bg-zinc-700'
            }`}
          >
            <span class="w-3.5 h-3.5 flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconHistory} />
            <span>tidy undo</span>
          </button>
        </div>
      </div>

      {/* Info Status Bar */}
      <div class="border-b border-zinc-800 bg-zinc-950/60 px-4 py-2 text-xs font-mono flex flex-wrap items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <span class="inline-block h-2 w-2 rounded-full bg-cyan-400" />
          <span class="text-zinc-400">
            {isOrganized() ? (
              <span class="text-cyan-400">
                State: Structured (5 Category subdirectories, 0 collisions unresolved)
              </span>
            ) : (
              <span class="text-amber-400">
                State: Cluttered (8 files in root, unorganized)
              </span>
            )}
          </span>
        </div>
        <div class="text-zinc-500">
          Ledger: <span class="text-zinc-400">~/.local/state/tidy/history.db</span>
        </div>
      </div>

      {/* Category Filter Tabs */}
      <div class="flex items-center gap-1 border-b border-zinc-800 bg-zinc-900 px-4 py-2 overflow-x-auto text-xs font-mono">
        <button
          onClick={() => setActiveTab('all')}
          class={`rounded px-2.5 py-1 transition-colors ${
            activeTab() === 'all'
              ? 'bg-zinc-800 text-zinc-100 font-semibold'
              : 'text-zinc-400 hover:text-zinc-200'
          }`}
        >
          All Items ({INITIAL_FILES.length})
        </button>
        <button
          onClick={() => setActiveTab('documents')}
          class={`rounded px-2.5 py-1 transition-colors ${
            activeTab() === 'documents'
              ? 'bg-zinc-800 text-zinc-100 font-semibold'
              : 'text-zinc-400 hover:text-zinc-200'
          }`}
        >
          Documents (2)
        </button>
        <button
          onClick={() => setActiveTab('images')}
          class={`rounded px-2.5 py-1 transition-colors ${
            activeTab() === 'images'
              ? 'bg-zinc-800 text-zinc-100 font-semibold'
              : 'text-zinc-400 hover:text-zinc-200'
          }`}
        >
          Images (2)
        </button>
        <button
          onClick={() => setActiveTab('archives')}
          class={`rounded px-2.5 py-1 transition-colors ${
            activeTab() === 'archives'
              ? 'bg-zinc-800 text-zinc-100 font-semibold'
              : 'text-zinc-400 hover:text-zinc-200'
          }`}
        >
          Archives (1)
        </button>
        <button
          onClick={() => setActiveTab('code')}
          class={`rounded px-2.5 py-1 transition-colors ${
            activeTab() === 'code'
              ? 'bg-zinc-800 text-zinc-100 font-semibold'
              : 'text-zinc-400 hover:text-zinc-200'
          }`}
        >
          Code (1)
        </button>
      </div>

      {/* File List Table */}
      <div class="overflow-x-auto">
        <table class="w-full text-left font-mono text-xs">
          <thead>
            <tr class="border-b border-zinc-800 bg-zinc-950/40 text-zinc-400">
              <th class="py-2.5 px-4 font-medium">Path & Name</th>
              <th class="py-2.5 px-4 font-medium">Category / Destination</th>
              <th class="py-2.5 px-4 font-medium">Size</th>
              <th class="py-2.5 px-4 font-medium">Safety Status</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-zinc-800/60">
            <For each={filteredFiles()}>
              {(file) => (
                <tr class="hover:bg-zinc-850/50 transition-colors">
                  <td class="py-2.5 px-4">
                    <div class="flex items-center gap-2">
                      <span class="w-4 h-4 text-zinc-500 shrink-0 flex items-center justify-center [&>svg]:w-4 [&>svg]:h-4" innerHTML={iconFile} />
                      <div>
                        <div class="font-medium text-zinc-200">
                          {isOrganized() ? file.organizedPath : file.originalPath}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td class="py-2.5 px-4">
                    <span class="inline-flex items-center rounded border border-zinc-800 bg-zinc-950 px-2 py-0.5 text-[11px] text-zinc-300">
                      {isOrganized() ? (file.isDownloadGuard ? 'Protected in Root' : file.category + '/' + file.ext.toUpperCase()) : 'Downloads (root)'}
                    </span>
                  </td>
                  <td class="py-2.5 px-4 text-zinc-400">{file.size}</td>
                  <td class="py-2.5 px-4">
                    <Show when={file.isDownloadGuard}>
                      <span class="inline-flex items-center gap-1 rounded border border-amber-900/60 bg-amber-950/40 px-2 py-0.5 text-[11px] text-amber-300">
                        <span class="w-3 h-3 flex items-center justify-center [&>svg]:w-3 [&>svg]:h-3" innerHTML={iconAlertTriangle} />
                        Download Guard
                      </span>
                    </Show>
                    <Show when={file.isCollision}>
                      <span class={`inline-flex items-center gap-1 rounded px-2 py-0.5 text-[11px] ${
                        isOrganized()
                          ? 'border border-cyan-900/60 bg-cyan-950/40 text-cyan-300'
                          : 'border border-zinc-800 bg-zinc-950 text-zinc-400'
                      }`}>
                        {isOrganized() ? 'Renamed: (1).png' : 'Collision Risk'}
                      </span>
                    </Show>
                    <Show when={!file.isDownloadGuard && !file.isCollision}>
                      <span class="inline-flex items-center gap-1 text-zinc-400 text-[11px]">
                        <span class="w-3 h-3 text-cyan-400 flex items-center justify-center [&>svg]:w-3 [&>svg]:h-3" innerHTML={iconCheck} />
                        {isOrganized() ? 'Moved & Logged' : 'Ready'}
                      </span>
                    </Show>
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>

      {/* Terminal Footer Console Log */}
      <Show when={lastAction()}>
        <div class="border-t border-zinc-800 bg-zinc-950 px-4 py-3 font-mono text-xs text-zinc-400 flex items-start gap-2">
          <span class="text-cyan-400 shrink-0">$</span>
          <span>{lastAction()}</span>
        </div>
      </Show>
    </div>
  );
}
