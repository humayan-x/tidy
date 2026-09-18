import { createSignal, For } from 'solid-js';

type ViewMode = 'toml' | 'categories';

const CATEGORIES = [
  {
    name: 'Documents',
    dest: 'Documents/<EXT>',
    count: '16 formats',
    examples: ['pdf', 'docx', 'xlsx', 'pptx', 'csv', 'epub', 'md', 'txt', 'rtf', 'odt'],
  },
  {
    name: 'Images',
    dest: 'Images/<EXT>',
    count: '12 formats',
    examples: ['png', 'jpg', 'jpeg', 'webp', 'svg', 'gif', 'bmp', 'tiff', 'heic', 'avif'],
  },
  {
    name: 'Archives',
    dest: 'Archives/<EXT>',
    count: '14 formats',
    examples: ['zip', 'tar.gz', 'tar.bz2', 'tar.xz', 'tar.zst', '7z', 'rar', 'gz', 'bz2'],
    highlight: 'Compound archive detection',
  },
  {
    name: 'Code',
    dest: 'Code/<EXT>',
    count: '18 formats',
    examples: ['rs', 'ts', 'js', 'py', 'go', 'c', 'cpp', 'java', 'html', 'css', 'json', 'toml'],
  },
  {
    name: 'Audio',
    dest: 'Audio/<EXT>',
    count: '9 formats',
    examples: ['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a', 'wma'],
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

# Whether to organize files into extension-based subfolders (e.g. Documents/PDF, Images/PNG)
nest_by_extension = true

# Download and temporary file guard patterns to ignore during watch/run
ignore_patterns = [
  "*.crdownload",
  "*.part",
  "*.download",
  "*.aria2",
  "*.tmp",
  "*.swp"
]

[categories.Documents]
destination = "Docs"
extensions = ["pdf", "docx", "xlsx", "pptx", "txt", "md", "csv"]

[categories.Archives]
destination = "Archives"
extensions = ["zip", "tar.gz", "tar.bz2", "tar.xz", "tar.zst", "7z", "rar"]`;

export default function ConfigExplorer() {
  const [viewMode, setViewMode] = createSignal<ViewMode>('toml');

  return (
    <div class="rounded-lg border border-zinc-800 bg-zinc-900 overflow-hidden shadow-2xl">
      {/* View Switcher Header */}
      <div class="flex items-center justify-between border-b border-zinc-800 bg-zinc-950 px-4 py-3">
        <div class="flex items-center gap-2">
          <div class="flex gap-1.5">
            <div class="h-3 w-3 rounded-full bg-zinc-700" />
            <div class="h-3 w-3 rounded-full bg-zinc-700" />
            <div class="h-3 w-3 rounded-full bg-zinc-700" />
          </div>
          <span class="ml-2 font-mono text-xs text-zinc-400">
            {viewMode() === 'toml' ? '~/.config/tidy/config.toml' : 'Zero-Config Category Classification Matrix'}
          </span>
        </div>

        <div class="flex items-center rounded border border-zinc-800 bg-zinc-900 p-0.5">
          <button
            onClick={() => setViewMode('toml')}
            class={`rounded px-3 py-1 font-mono text-xs transition-colors ${
              viewMode() === 'toml'
                ? 'bg-zinc-800 text-zinc-100 font-semibold'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            config.toml
          </button>
          <button
            onClick={() => setViewMode('categories')}
            class={`rounded px-3 py-1 font-mono text-xs transition-colors ${
              viewMode() === 'categories'
                ? 'bg-zinc-800 text-zinc-100 font-semibold'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            70+ Formats Matrix
          </button>
        </div>
      </div>

      {/* Content Area */}
      {viewMode() === 'toml' ? (
        <div class="p-4 sm:p-6 overflow-x-auto bg-zinc-950">
          <pre class="font-mono text-xs leading-relaxed text-zinc-300">
            <code>{TOML_CONTENT}</code>
          </pre>
        </div>
      ) : (
        <div class="p-4 sm:p-6 bg-zinc-900">
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            <For each={CATEGORIES}>
              {(cat) => (
                <div class="rounded border border-zinc-800 bg-zinc-950 p-4 space-y-2.5">
                  <div class="flex items-center justify-between">
                    <span class="font-mono text-xs font-semibold text-zinc-100">{cat.name}</span>
                    <span class="rounded border border-zinc-800 bg-zinc-900 px-1.5 py-0.5 font-mono text-[10px] text-zinc-400">
                      {cat.count}
                    </span>
                  </div>
                  <div class="text-[11px] font-mono text-cyan-400">
                    Target: <span class="text-zinc-300">{cat.dest}</span>
                  </div>
                  <div class="flex flex-wrap gap-1 pt-1">
                    <For each={cat.examples}>
                      {(ext) => (
                        <span class="rounded bg-zinc-900 border border-zinc-800/80 px-1.5 py-0.5 font-mono text-[10px] text-zinc-400">
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
      <div class="border-t border-zinc-800 bg-zinc-950 px-4 py-2.5 font-mono text-xs text-zinc-500 flex items-center justify-between">
        <span>Generate scaffold via: <code class="text-zinc-300">tidy init</code></span>
        <span>Magic-byte fallback via <code class="text-zinc-300">infer</code></span>
      </div>
    </div>
  );
}
