

tcp_server_test0:
	@ cargo run --bin tcp_server_test0

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
