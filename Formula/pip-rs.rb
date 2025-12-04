class PipRs < Formula
  desc "High-performance Rust implementation of pip - the Python package installer"
  homepage "https://github.com/yingkitw/pip-rs"
  url "https://github.com/yingkitw/pip-rs/archive/refs/tags/v1.0.0-rc.1.tar.gz"
  sha256 "d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed"
  license "Apache-2.0"
  head "https://github.com/yingkitw/pip-rs.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/pip-rs", "--help"
  end
end
