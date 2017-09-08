Vagrant.configure(2) do |config|
  config.vm.box = 'debian/jessie64'

  config.vm.provision "shell", inline: <<-EOS
    apt-get -y update
    apt-get install -y clang git vim curl
  EOS

  config.vm.provision "shell", privileged: false, inline: <<-EOS
    curl https://sh.rustup.rs -sSf > install-rustup
    chmod +x install-rustup
    ./install-rustup -y
    source ~/.cargo/env
    rustup target add x86_64-unknown-linux-musl
    cargo install -f just
    git clone https://github.com/casey/just.git
  EOS
end
