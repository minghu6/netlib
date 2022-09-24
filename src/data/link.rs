use crate::defraw;


defraw! {
    pub struct rtnl_link_stats {
        rx_packets: u32,
        tx_packets: u32,
        rx_bytes: u32,
        tx_bytes: u32,
        rx_errors: u32,
        tx_errors: u32,
        rx_dropped: u32,
        tx_dropped: u32,
        multicast: u32,
        collisions: u32,
        /* detailed rx_errors: */
        rx_length_errors: u32,
        rx_over_errors: u32,
        rx_crc_errors: u32,
        rx_frame_errors: u32,
        rx_fifo_errors: u32,
        rx_missed_errors: u32,
        /* detailed tx_errors */
        tx_aborted_errors: u32,
        tx_carrier_errors: u32,
        tx_fifo_errors: u32,
        tx_heartbeat_errors: u32,
        tx_window_errors: u32,
        /* for cslip etc */
        rx_compressed: u32,
        tx_compressed: u32,
        rx_nohandler: u32,
    }
}

