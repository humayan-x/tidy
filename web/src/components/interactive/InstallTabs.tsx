import { createSignal, For, Show } from 'solid-js';
import iconCopy from '@tabler/icons/outline/copy.svg?raw';
import iconCheck from '@tabler/icons/outline/check.svg?raw';

type TabKey = 'curl' | 'npx' | 'brew' | 'aur' | 'cargo';

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
    subtext: 'Run immediately without prior installation, or install globally via: npm install -g @humayan-x/tidy',
  },
  {
    key: 'brew',
    label: 'Homebrew',
    command: 'brew install humayan-x/tap/tidy',
    subtext: 'Official Homebrew tap for macOS and Linuxbrew systems.',
  },
  {
    key: 'aur',
    label: 'Arch Linux (AUR)',
    command: 'yay -S tidy-bin',
    subtext: 'Pre-built binary package available on the Arch User Repository.',
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
    <div class="rounded-lg border border-zinc-800 bg-zinc-900 overflow-hidden shadow-xl">
      {/* Tabs Header */}
      <div class="flex items-center border-b border-zinc-800 bg-zinc-950 overflow-x-auto">
        <For each={OPTIONS}>
          {(option) => (
            <button
              onClick={() => setActiveTab(option.key)}
              class={`border-r border-zinc-800 px-4 py-3 font-mono text-xs font-medium transition-colors whitespace-nowrap ${
                activeTab() === option.key
                  ? 'bg-zinc-900 text-zinc-100 border-b-2 border-b-cyan-400'
                  : 'text-zinc-400 hover:bg-zinc-900/50 hover:text-zinc-200'
              }`}
            >
              {option.label}
            </button>
          )}
        </For>
      </div>

      {/* Terminal Command Area */}
      <div class="p-4 sm:p-6 bg-zinc-900">
        <div class="flex items-center justify-between gap-4 rounded border border-zinc-800 bg-zinc-950 px-4 py-3.5 font-mono text-xs sm:text-sm">
          <div class="flex items-center gap-2.5 overflow-x-auto py-1">
            <span class="text-cyan-400 select-none">$</span>
            <span class="text-zinc-200 select-all whitespace-nowrap">
              {currentOption().command}
            </span>
          </div>

          <button
            onClick={handleCopy}
            class="flex items-center gap-1.5 rounded border border-zinc-700 bg-zinc-800 px-3 py-1.5 font-mono text-xs text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 shrink-0"
            title="Copy command"
          >
            <Show
              when={copied()}
              fallback={
                <>
                  <span class="w-3.5 h-3.5 text-zinc-400 flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconCopy} />
                  <span>Copy</span>
                </>
              }
            >
              <span class="w-3.5 h-3.5 text-cyan-400 flex items-center justify-center [&>svg]:w-3.5 [&>svg]:h-3.5" innerHTML={iconCheck} />
              <span class="text-cyan-400">Copied</span>
            </Show>
          </button>
        </div>

        <p class="mt-3 text-xs text-zinc-400">
          {currentOption().subtext}
        </p>
      </div>
    </div>
  );
}
