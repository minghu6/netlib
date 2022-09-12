
test_ping:
	@ cargo build --bin ping
	# @ sudo ./target/debug/ping 114.114.114.114
	# @ sudo ./target/debug/ping 110.242.68.66
	@ sudo ./target/debug/ping baidu.com
	# @ sudo ./target/debug/ping opensource.com
	# @ sudo ./target/debug/ping localhost


test_dos_icmp:
	@ cargo build --bin dos_icmp
	# @ sudo ./target/debug/dos_icmp baidu.com
	@ sudo ./target/debug/dos_icmp 110.242.68.66

test_dos_tcp:
	@ cargo build --bin dos_tcp
	@ sudo ./target/debug/dos_tcp 110.242.68.66

run_tcp_server_r0:
	@ cargo build --bin tcp_server_r0
	@ ./target/debug/tcp_server_r0

run_tcp_client_r0:
	@ cargo build --bin tcp_client_r0
	@ ./target/debug/tcp_client_r0

run_tcp_client_c0:
	@ cargo build --bin tcp_client_c0
	@ ./target/debug/tcp_client_c0

