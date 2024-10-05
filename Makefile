INSTALLED_DIR=$(HOME)/.cargo/bin
IF=wlp2s0

test_ping:
	# @ cargo build --example m6ping
	# @ sudo ./target/debug/examples/m6ping baidu.com
	@ cargo run --example m6ping -- tencent.com

install_ping:
	@ cargo install --path . --example m6ping
	# https://man7.org/linux/man-pages/man7/capabilities.7.html
	# e effective, i inheritable, p permitted
	# remove `nosuid` partition mount option (/home on Ubuntu) to enable capability setting (after remount it)
	# or move exeecuable file to other partiation
	# or run with sudo
	@ sudo setcap CAP_NET_RAW=epi $(INSTALLED_DIR)/m6ping

test_dos_icmp:
	@ cargo run --example dos_icmp -- 110.242.68.66

test_dos_tcp:
	@ cargo run --example dos_tcp -- 110.242.68.66

run_tcp_server_r0:
	@ cargo run --example tcp_server_r0

run_tcp_client_r0:
	@ cargo run --example tcp_client_r0

run_tcp_client_c0:
	@ cargo run --example tcp_client_c0

run_shttpd:
	@ cargo run --example shttpd -- -c res/shttpd/shttpd.user.yaml

run_arp:
	@ cargo run --example arp -- baidu.com

run_sip:
	@ cargo build --example sip
	@ sudo setcap CAP_NET_RAW=epi ./target/debug/examples/sip
	@ ./target/debug/examples/sip -- $(IF)

setup_dev:
	@ sudo ip tuntap add dev tunx mode tun
	@ sudo ip addr add 10.0.0.1/24 dev tunx
	@ sudo ip link set tunx up

test_dev:
	@ cargo test test_tun -- --nocapture
