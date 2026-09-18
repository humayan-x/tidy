import { createSignal, For, Show } from 'solid-js';
import iconCopy from '@tabler/icons/outline/copy.svg?raw';
import iconCheck from '@tabler/icons/outline/check.svg?raw';

type TabKey = 'curl' | 'npx' | 'cargo';

interface InstallOption {
  key: TabKey;
  label: string;
  command: string;
  subtext: string;
}

const OPTIONS: InstallOption[] = [
  {
    key: 'curl',
    label: 'curl (Shell Script)',
    command: 'curl -fsSL https://raw.githubusercontent.com/humayan-x/tidy/main/install.sh | sh',
    subtext: 'Fetches pre-compiled static binary for Linux (x86_64 / aarch64) or macOS (Apple Silicon / Intel).',
  },
  {
    key: 'npx',
    label: 'npx / npm',
    command: 'npx @humayan-x/tidy run --dry-run',
    subtext: 'Run immediately without prior installation, or install globally via npm (use --allow-scripts on npm 10.9+).',
  },
  {
    key: 'cargo',
    label: 'Cargo (Source)',
    command: 'cargo install --git https://github.com/humayan-x/tidy.git',
    subtext: 'Requires Rust 1.75+ toolchain. Compiles with link-time optimization (LTO).',
  },
];

export default function InstallTabs() {
  const [activeTab, setActiveTab] = createSignal<TabKey>('curl');
  const [copied, setCopied] = createSignal(false);

  const currentOption = () => OPTIONS.find((o) => o.key === activeTab()) || OPTIONS[0];

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(currentOption().command);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback
    }
  };

  return (
    <div class="rounded-lg border border-line-default bg-surface-card overflow-hidden shadow-xl">
      {/* Tabs Header */}
      <div class="flex items-center border-b border-line-default bg-surface-root overflow-x-auto no-scrollbar touch-scroll">
        <For each={OPTIONS}>
          {(option) => (
            <button
              onClick={() => setActiveTab(option.key)}
              class={`border-r border-line-default px-3.5 sm:px-4 py-2.5 sm:py-3 font-mono text-xs font-medium transition-colors whitespace-nowrap min-h-[42px] ${
                activeTab() === option.key
                  ? 'bg-surface-card text-title border-b-2 border-b-primary'
                  : 'text-muted hover:bg-surface-subtle hover:text-title'
              }`}
            >
              {option.label}
            </button>
          )}
        </For>
      </div>

      {/* Terminal Command Area */}
      <div class="p-3 sm:p-6 bg-surface-card">
        <div class="flex items-center justify-between gap-3 rounded border border-line-default bg-surface-root px-3 py-2.5 sm:px-4 sm:py-3.5 font-mono text-xs sm:text-sm">
          <div class="flex items-center gap-2 sm:gap-2.5 overflow-x-auto no-scrollbar touch-scroll py-1 text-left">
            <span class="text-primary select-none">$</span>
            <span class="text-title select-all whitespace-nowrap text-[11px] sm:text-sm">
              {currentOption().command}
            </span>
          </div>

          <button
            onClick={handleCopy}
            class="flex items-center gap-1.5 rounded border border-line-strong bg-surface-elevated px-2.5 sm:px-3 py-1.5 font-mono text-xs text-body transition-colors hover:bg-surface-hover hover:text-title shrink-0 min-h-[30px]"
            title="Copy command"
          >
            <Show
              when={copied()}
              fallback={
                <>
                  <span class="w-3.5 h-3.5 text-muted flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconCopy} />
                  <span>Copy</span>
                </>
              }
            >
              <span class="w-3.5 h-3.5 text-primary flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconCheck} />
              <span class="text-primary">Copied</span>
            </Show>
          </button>
        </div>

        <p class="mt-2.5 sm:mt-3 text-[11px] sm:text-xs text-muted">
          {currentOption().subtext}
        </p>
      </div>
    </div>
  );
}
