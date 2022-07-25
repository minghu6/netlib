

tcp_server_test0:
	@ cargo run --bin tcp_server_test0

test_ping:
	@ cargo build --bin ping
	@ sudo ./target/debug/ping 114.114.114.114

