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
    <div class="rounded-lg border border-line-default bg-surface-card overflow-hidden shadow-2xl">
      {/* Terminal Titlebar */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-line-default bg-surface-root px-3.5 sm:px-4 py-3">
        <div class="flex items-center gap-2">
          <div class="flex gap-1.5 shrink-0">
            <div class="h-3 w-3 rounded-full bg-surface-hover" />
            <div class="h-3 w-3 rounded-full bg-surface-hover" />
            <div class="h-3 w-3 rounded-full bg-surface-hover" />
          </div>
          <span class="ml-1.5 font-mono text-[11px] sm:text-xs text-muted truncate">
            tidy-simulator — {isOrganized() ? '~/Downloads (Organized Tree)' : '~/Downloads (Unsorted Flat)'}
          </span>
        </div>

        {/* Action Buttons */}
        <div class="flex items-center gap-2 w-full sm:w-auto">
          <button
            onClick={handleOrganize}
            disabled={isOrganized()}
            class={`flex-1 sm:flex-none inline-flex items-center justify-center gap-1.5 rounded px-3 py-1.5 sm:py-1 font-mono text-xs font-medium transition-colors min-h-[36px] ${
              isOrganized()
                ? 'border border-line-default bg-surface-elevated/50 text-dim cursor-not-allowed'
                : 'border border-primary/60 bg-primary/10 text-primary hover:bg-primary/20'
            }`}
          >
            <span class="w-3.5 h-3.5 flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconTerminal} />
            <span>Run tidy</span>
          </button>

          <button
            onClick={handleUndo}
            disabled={!isOrganized()}
            class={`flex-1 sm:flex-none inline-flex items-center justify-center gap-1.5 rounded px-3 py-1.5 sm:py-1 font-mono text-xs font-medium transition-colors min-h-[36px] ${
              !isOrganized()
                ? 'border border-line-default bg-surface-elevated/50 text-dim cursor-not-allowed'
                : 'border border-line-strong bg-surface-elevated text-title hover:bg-surface-hover'
            }`}
          >
            <span class="w-3.5 h-3.5 flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconHistory} />
            <span>tidy undo</span>
          </button>
        </div>
      </div>

      {/* Info Status Bar */}
      <div class="border-b border-line-default bg-surface-subtle px-3.5 sm:px-4 py-2 text-[11px] sm:text-xs font-mono flex flex-col sm:flex-row sm:items-center justify-between gap-1.5 sm:gap-2">
        <div class="flex items-center gap-2">
          <span class="inline-block h-2 w-2 rounded-full bg-primary shrink-0" />
          <span class="text-muted">
            {isOrganized() ? (
              <span class="text-primary">
                State: Structured (5 Category subdirectories, 0 collisions unresolved)
              </span>
            ) : (
              <span class="text-amber-400">
                State: Cluttered (8 files in root, unorganized)
              </span>
            )}
          </span>
        </div>
        <div class="text-dim text-[10px] sm:text-xs">
          Ledger: <span class="text-muted">~/.local/state/tidy/history.db</span>
        </div>
      </div>

      {/* Category Filter Tabs */}
      <div class="flex items-center gap-1 border-b border-line-default bg-surface-card px-3 sm:px-4 py-2 overflow-x-auto no-scrollbar touch-scroll text-xs font-mono">
        <button
          onClick={() => setActiveTab('all')}
          class={`rounded px-2.5 py-1 transition-colors shrink-0 ${
            activeTab() === 'all'
              ? 'bg-surface-elevated text-title font-semibold'
              : 'text-muted hover:text-title'
          }`}
        >
          All Items ({INITIAL_FILES.length})
        </button>
        <button
          onClick={() => setActiveTab('documents')}
          class={`rounded px-2.5 py-1 transition-colors shrink-0 ${
            activeTab() === 'documents'
              ? 'bg-surface-elevated text-title font-semibold'
              : 'text-muted hover:text-title'
          }`}
        >
          Documents (2)
        </button>
        <button
          onClick={() => setActiveTab('images')}
          class={`rounded px-2.5 py-1 transition-colors shrink-0 ${
            activeTab() === 'images'
              ? 'bg-surface-elevated text-title font-semibold'
              : 'text-muted hover:text-title'
          }`}
        >
          Images (2)
        </button>
        <button
          onClick={() => setActiveTab('archives')}
          class={`rounded px-2.5 py-1 transition-colors shrink-0 ${
            activeTab() === 'archives'
              ? 'bg-surface-elevated text-title font-semibold'
              : 'text-muted hover:text-title'
          }`}
        >
          Archives (1)
        </button>
        <button
          onClick={() => setActiveTab('code')}
          class={`rounded px-2.5 py-1 transition-colors shrink-0 ${
            activeTab() === 'code'
              ? 'bg-surface-elevated text-title font-semibold'
              : 'text-muted hover:text-title'
          }`}
        >
          Code (1)
        </button>
      </div>

      {/* Mobile Card List View (<sm) */}
      <div class="block sm:hidden divide-y divide-line-subtle font-mono text-xs">
        <For each={filteredFiles()}>
          {(file) => (
            <div class="p-3 space-y-2 hover:bg-surface-subtle transition-colors">
              <div class="flex items-start gap-2">
                <span class="w-4 h-4 text-dim shrink-0 mt-0.5 flex items-center justify-center [&>svg]:w-4 [&>svg]:h-4" innerHTML={iconFile} />
                <div class="font-medium text-title break-all text-[11px] leading-snug">
                  {isOrganized() ? file.organizedPath : file.originalPath}
                </div>
              </div>
              <div class="flex flex-wrap items-center justify-between gap-2 pt-0.5 text-[11px]">
                <span class="inline-flex items-center rounded border border-line-default bg-surface-root px-2 py-0.5 text-body">
                  {isOrganized() ? (file.isDownloadGuard ? 'Protected in Root' : file.category + '/' + file.ext.toUpperCase()) : 'Downloads (root)'}
                </span>
                <div class="flex items-center gap-2">
                  <span class="text-muted text-[10px]">{file.size}</span>
                  <Show when={file.isDownloadGuard}>
                    <span class="inline-flex items-center gap-1 rounded border border-amber-900/60 bg-amber-950/40 px-1.5 py-0.5 text-[10px] text-amber-300">
                      <span class="w-2.5 h-2.5 flex items-center justify-center [&>svg]:w-2.5 [&>svg]:h-2.5" innerHTML={iconAlertTriangle} />
                      Guard
                    </span>
                  </Show>
                  <Show when={file.isCollision}>
                    <span class={`inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] ${
                      isOrganized()
                        ? 'border border-primary/40 bg-primary/10 text-primary'
                        : 'border border-line-default bg-surface-root text-muted'
                    }`}>
                      {isOrganized() ? 'Renamed: (1)' : 'Collision'}
                    </span>
                  </Show>
                  <Show when={!file.isDownloadGuard && !file.isCollision}>
                    <span class="inline-flex items-center gap-1 text-muted text-[10px]">
                      <span class="w-2.5 h-2.5 text-primary flex items-center justify-center [&>svg]:w-2.5 [&>svg]:h-2.5" innerHTML={iconCheck} />
                      {isOrganized() ? 'Moved' : 'Ready'}
                    </span>
                  </Show>
                </div>
              </div>
            </div>
          )}
        </For>
      </div>

      {/* Desktop File List Table (>=sm) */}
      <div class="hidden sm:block overflow-x-auto no-scrollbar touch-scroll">
        <table class="w-full text-left font-mono text-xs">
          <thead>
            <tr class="border-b border-line-default bg-surface-subtle text-muted">
              <th class="py-2.5 px-4 font-medium">Path & Name</th>
              <th class="py-2.5 px-4 font-medium">Category / Destination</th>
              <th class="py-2.5 px-4 font-medium">Size</th>
              <th class="py-2.5 px-4 font-medium">Safety Status</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-line-subtle">
            <For each={filteredFiles()}>
              {(file) => (
                <tr class="hover:bg-surface-subtle transition-colors">
                  <td class="py-2.5 px-4">
                    <div class="flex items-center gap-2">
                      <span class="w-4 h-4 text-dim shrink-0 flex items-center justify-center [&>svg]:w-4 [&>svg]:h-4" innerHTML={iconFile} />
                      <div>
                        <div class="font-medium text-title">
                          {isOrganized() ? file.organizedPath : file.originalPath}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td class="py-2.5 px-4">
                    <span class="inline-flex items-center rounded border border-line-default bg-surface-root px-2 py-0.5 text-[11px] text-body">
                      {isOrganized() ? (file.isDownloadGuard ? 'Protected in Root' : file.category + '/' + file.ext.toUpperCase()) : 'Downloads (root)'}
                    </span>
                  </td>
                  <td class="py-2.5 px-4 text-muted">{file.size}</td>
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
                          ? 'border border-primary/40 bg-primary/10 text-primary'
                          : 'border border-line-default bg-surface-root text-muted'
                      }`}>
                        {isOrganized() ? 'Renamed: (1).png' : 'Collision Risk'}
                      </span>
                    </Show>
                    <Show when={!file.isDownloadGuard && !file.isCollision}>
                      <span class="inline-flex items-center gap-1 text-muted text-[11px]">
                        <span class="w-3 h-3 text-primary flex items-center justify-center [&>svg]:w-3 [&>svg]:h-3" innerHTML={iconCheck} />
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
        <div class="border-t border-line-default bg-surface-root px-3.5 sm:px-4 py-3 font-mono text-[11px] sm:text-xs text-muted flex items-start gap-2">
          <span class="text-primary shrink-0">$</span>
          <span>{lastAction()}</span>
        </div>
      </Show>
    </div>
  );
}
