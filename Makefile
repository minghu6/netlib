INSTALLED_DIR=$(HOME)/.cargo/bin


test_ping:
	# @ cargo build --example m6ping
	# @ sudo ./target/debug/examples/m6ping baidu.com
	@ cargo run --example m6ping -- tencent.com

install_ping:
	@ cargo install --path . --example m6ping
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
	@ cargo run --example sip -- wlp3s0

test:
	@ cargo test test_tun -- --nocapture
