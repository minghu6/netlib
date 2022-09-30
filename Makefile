
test_ping:
	@ cargo build --example ping
	@ sudo ./target/debug/examples/ping baidu.com
	# @ sudo ./target/debug/examples/ping opensource.com
	# @ sudo ./target/debug/examples/ping localhost
	@ cargo build --example ping -- baidu.com


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
