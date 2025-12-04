class PipRs < Formula
  desc "High-performance Rust implementation of pip - the Python package installer"
  homepage "https://github.com/yourusername/pip-rs"
  url "https://github.com/yourusername/pip-rs/archive/refs/tags/v1.0.0-rc.1.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "Apache-2.0"
  head "https://github.com/yourusername/pip-rs.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "pip-rs", shell_output("#{bin}/pip --version")
  end
end
