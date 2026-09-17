class Tidy < Formula
  desc "A blazing-fast, zero-dependency, local-first file organizer and watcher in Rust"
  homepage "https://github.com/humayan-x/tidy"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/humayan-x/tidy/releases/download/v#{version}/tidy-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_APPLE_DARWIN_SHA256"
    else
      url "https://github.com/humayan-x/tidy/releases/download/v#{version}/tidy-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_APPLE_DARWIN_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/humayan-x/tidy/releases/download/v#{version}/tidy-v#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_X86_64_UNKNOWN_LINUX_MUSL_SHA256"
    end
  end

  def install
    bin.install "tidy"

    # Generate and install shell completions
    (bash_completion/"tidy").write Utils.safe_popen_read("#{bin}/tidy", "completions", "bash")
    (zsh_completion/"_tidy").write Utils.safe_popen_read("#{bin}/tidy", "completions", "zsh")
    (fish_completion/"tidy.fish").write Utils.safe_popen_read("#{bin}/tidy", "completions", "fish")
  end

  service do
    run [opt_bin/"tidy", "watch"]
    keep_alive true
    working_dir Dir.home
    log_path var/"log/tidy.log"
    error_log_path var/"log/tidy.err"
  end

  test do
    assert_match "tidy #{version}", shell_output("#{bin}/tidy --version")
  end
end
