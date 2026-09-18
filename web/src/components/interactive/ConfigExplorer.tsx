import { createSignal, For } from 'solid-js';

type ViewMode = 'toml' | 'categories';

const CATEGORIES = [
  {
    name: 'Documents',
    dest: 'Documents/{PDFs,Word,Excel,...}',
    count: '20 formats',
    examples: ['pdf', 'docx', 'xlsx', 'pptx', 'csv', 'epub', 'md', 'txt', 'rtf', 'odt'],
    highlight: 'Curated suites (Word, Excel, PDFs)',
  },
  {
    name: 'Images',
    dest: 'Images/<EXT>',
    count: '16 formats',
    examples: ['png', 'jpg', 'jpeg', 'webp', 'svg', 'gif', 'bmp', 'tiff', 'heic', 'avif'],
  },
  {
    name: 'Archives',
    dest: 'Archives/<EXT>',
    count: '19 formats',
    examples: ['zip', 'tar.gz', 'tar.bz2', 'tar.xz', 'tar.zst', '7z', 'rar', 'gz', 'bz2'],
    highlight: 'Compound archive detection',
  },
  {
    name: 'Code',
    dest: 'Code/<EXT>',
    count: '34 formats',
    examples: ['rs', 'ts', 'js', 'py', 'go', 'c', 'cpp', 'java', 'html', 'css', 'json', 'toml'],
  },
  {
    name: 'Audio',
    dest: 'Audio/<EXT>',
    count: '12 formats',
    examples: ['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a', 'wma', 'opus'],
  },
  {
    name: 'Video',
    dest: 'Video/<EXT>',
    count: '10 formats',
    examples: ['mp4', 'mkv', 'mov', 'avi', 'webm', 'flv', 'wmv'],
  },
];

const TOML_CONTENT = `[settings]
# Default directory to watch when running 'tidy watch' without --path
# watch_dir = "/home/user/Downloads"

# Whether to ignore hidden files and directories (starting with '.')
ignore_hidden = true

# Debounce interval in milliseconds
debounce_ms = 2000

# Polling interval in milliseconds for write-completion stability
stability_tick_ms = 500

# Whether to scan or watch subdirectories recursively
recursive = false

# Whether to organize files into extension-based subfolders (e.g. Documents/PDFs, Documents/Word, Images/PNG)
nest_by_extension = true

# Download and temporary file guard patterns to ignore during watch/run
ignore_patterns = [
  "*.crdownload",
  "*.part",
  "*.download",
  "*.aria2",
  "*.tmp",
  "*.partial",
  ".DS_Store",
  "Thumbs.db",
  "*.swp",
  "*~"
]

# Map category names to custom folder names (optional overrides)
[destinations]
Images = "Photos"
Documents = "Docs"

# Custom subfolder overrides for nested extension folders (optional)
# Defaults: .pdf -> PDFs, .doc/.docx -> Word, .xls/.xlsx -> Excel, .ppt/.pptx -> PowerPoint
# All unlisted extensions default to uppercase (e.g. .png -> Images/PNG)
[subfolders]
odt = "Word"
rtf = "Word"

# User-defined custom categories (optional; tidy includes 70+ default formats)
[categories]
3D = ["obj", "stl", "blend", "fbx"]
EBooks = ["epub", "mobi"]
Data = ["csv", "tsv", "parquet", "jsonl"]`;

export default function ConfigExplorer() {
  const [viewMode, setViewMode] = createSignal<ViewMode>('toml');

  return (
    <div class="rounded-lg border border-line-default bg-surface-card overflow-hidden shadow-2xl">
      {/* View Switcher Header */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5 border-b border-line-default bg-surface-root px-3.5 sm:px-4 py-3">
        <div class="flex items-center gap-2">
          <div class="flex gap-1.5 shrink-0">
            <div class="h-3 w-3 rounded-full bg-surface-hover" />
            <div class="h-3 w-3 rounded-full bg-surface-hover" />
            <div class="h-3 w-3 rounded-full bg-surface-hover" />
          </div>
          <span class="ml-1.5 font-mono text-[11px] sm:text-xs text-muted truncate">
            {viewMode() === 'toml' ? '~/.config/tidy/config.toml' : 'Zero-Config Category Classification Matrix'}
          </span>
        </div>

        <div class="flex items-center rounded border border-line-default bg-surface-card p-0.5 w-full sm:w-auto">
          <button
            onClick={() => setViewMode('toml')}
            class={`flex-1 sm:flex-none rounded px-3 py-1.5 sm:py-1 font-mono text-xs transition-colors text-center ${
              viewMode() === 'toml'
                ? 'bg-surface-elevated text-title font-semibold'
                : 'text-muted hover:text-title'
            }`}
          >
            config.toml
          </button>
          <button
            onClick={() => setViewMode('categories')}
            class={`flex-1 sm:flex-none rounded px-3 py-1.5 sm:py-1 font-mono text-xs transition-colors text-center ${
              viewMode() === 'categories'
                ? 'bg-surface-elevated text-title font-semibold'
                : 'text-muted hover:text-title'
            }`}
          >
            70+ Formats Matrix
          </button>
        </div>
      </div>

      {/* Content Area */}
      {viewMode() === 'toml' ? (
        <div class="p-3 sm:p-6 overflow-x-auto no-scrollbar touch-scroll bg-surface-root">
          <pre class="font-mono text-[11px] sm:text-xs leading-relaxed text-body">
            <code>{TOML_CONTENT}</code>
          </pre>
        </div>
      ) : (
        <div class="p-3 sm:p-6 bg-surface-card">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            <For each={CATEGORIES}>
              {(cat) => (
                <div class="rounded border border-line-default bg-surface-root p-3.5 sm:p-4 space-y-2">
                  <div class="flex items-center justify-between">
                    <span class="font-mono text-xs font-semibold text-title">{cat.name}</span>
                    <span class="font-mono text-[11px] text-dim">
                      {cat.count}
                    </span>
                  </div>
                  <div class="text-[11px] font-mono text-primary">
                    Target: <span class="text-body">{cat.dest}</span>
                  </div>
                  <div class="flex flex-wrap gap-1.5 pt-1">
                    <For each={cat.examples}>
                      {(ext) => (
                        <span class="font-mono text-[11px] text-muted">
                          .{ext}
                        </span>
                      )}
                    </For>
                  </div>
                  {cat.highlight && (
                    <div class="text-[10px] font-mono text-amber-400/90 pt-1">
                      {cat.highlight}
                    </div>
                  )}
                </div>
              )}
            </For>
          </div>
        </div>
      )}

      {/* Footer bar */}
      <div class="border-t border-line-default bg-surface-root px-3.5 sm:px-4 py-2.5 font-mono text-[10px] sm:text-xs text-dim flex flex-col sm:flex-row sm:items-center justify-between gap-1.5">
        <span>Generate scaffold: <code class="text-body">tidy init</code></span>
        <span>Magic-byte fallback via <code class="text-body">infer</code></span>
      </div>
    </div>
  );
}
