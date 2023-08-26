//! Module containing functions executed by the thread in charge of parsing sniffed packets and
//! inserting them in the shared map.

use std::sync::{Arc, Mutex};
use std::thread;

use etherparse::PacketHeaders;
use pcap::{Active, Capture};

use crate::countries::country_utils::mmdb_country_reader;
use crate::networking::manage_packets::{
    analyze_headers, get_address_to_lookup, modify_or_insert_in_map, reverse_dns_lookup,
};
use crate::networking::types::data_info::DataInfo;
use crate::networking::types::filters::Filters;
use crate::networking::types::info_address_port_pair::InfoAddressPortPair;
use crate::networking::types::my_device::MyDevice;
use crate::utils::asn::ASN_MMDB;
use crate::InfoTraffic;

/// The calling thread enters in a loop in which it waits for network packets, parses them according
/// to the user specified filters, and inserts them into the shared map variable.
pub fn parse_packets(
    current_capture_id: &Arc<Mutex<u16>>,
    device: &MyDevice,
    mut cap: Capture<Active>,
    filters: Filters,
    info_traffic_mutex: &Arc<Mutex<InfoTraffic>>,
    mmdb_country_path: String,
) {
    let capture_id = *current_capture_id.lock().unwrap();

    let country_db_readers = Arc::new(mmdb_country_reader(mmdb_country_path));
    let asn_db_reader = Arc::new(maxminddb::Reader::from_source(ASN_MMDB).unwrap());

    loop {
        match cap.next_packet() {
            Err(_) => {
                if *current_capture_id.lock().unwrap() != capture_id {
                    return;
                }
                continue;
            }
            Ok(packet) => {
                if *current_capture_id.lock().unwrap() != capture_id {
                    return;
                }
                match PacketHeaders::from_ethernet_slice(&packet) {
                    Err(_) => {
                        continue;
                    }
                    Ok(headers) => {
                        let mut exchanged_bytes = 0;
                        let mut mac_addresses = (String::new(), String::new());
                        let mut protocols = Filters::default();

                        let key_option = analyze_headers(
                            headers,
                            &mut mac_addresses,
                            &mut exchanged_bytes,
                            &mut protocols,
                        );
                        if key_option.is_none() {
                            continue;
                        }

                        let key = key_option.unwrap();
                        let mut new_info = InfoAddressPortPair::default();

                        let passed_filters = filters.matches(protocols);
                        if passed_filters {
                            new_info = modify_or_insert_in_map(
                                info_traffic_mutex,
                                &key,
                                device,
                                mac_addresses,
                                exchanged_bytes,
                                protocols.application,
                            );
                        }

                        let mut info_traffic = info_traffic_mutex
                            .lock()
                            .expect("Error acquiring mutex\n\r");
                        //increment number of sniffed packets and bytes
                        info_traffic.all_packets += 1;
                        info_traffic.all_bytes += exchanged_bytes;
                        // update dropped packets number
                        if let Ok(stats) = cap.stats() {
                            info_traffic.dropped_packets = stats.dropped;
                        }

                        if passed_filters {
                            info_traffic.add_packet(exchanged_bytes, new_info.traffic_direction);

                            // check the rDNS status of this address and act accordingly
                            let address_to_lookup =
                                get_address_to_lookup(&key, new_info.traffic_direction);
                            let r_dns_already_resolved = info_traffic
                                .addresses_resolved
                                .contains_key(&address_to_lookup);
                            let mut r_dns_waiting_resolution = false;
                            if !r_dns_already_resolved {
                                r_dns_waiting_resolution = info_traffic
                                    .addresses_waiting_resolution
                                    .contains_key(&address_to_lookup);
                            }

                            match (r_dns_waiting_resolution, r_dns_already_resolved) {
                                (false, false) => {
                                    // rDNS not requested yet (first occurrence of this address to lookup)

                                    // Add this address to the map of addresses waiting for a resolution
                                    // Useful to NOT perform again a rDNS lookup for this entry
                                    info_traffic.addresses_waiting_resolution.insert(
                                        address_to_lookup,
                                        DataInfo::new_with_first_packet(
                                            exchanged_bytes,
                                            new_info.traffic_direction,
                                        ),
                                    );

                                    // launch new thread to resolve host name
                                    let key2 = key.clone();
                                    let info_traffic2 = info_traffic_mutex.clone();
                                    let device2 = device.clone();
                                    let country_db_readers_2 = country_db_readers.clone();
                                    let asn_db_reader2 = asn_db_reader.clone();
                                    thread::Builder::new()
                                        .name("thread_reverse_dns_lookup".to_string())
                                        .spawn(move || {
                                            reverse_dns_lookup(
                                                &info_traffic2,
                                                &key2,
                                                new_info.traffic_direction,
                                                &device2,
                                                &country_db_readers_2,
                                                &asn_db_reader2,
                                            );
                                        })
                                        .unwrap();
                                }
                                (true, false) => {
                                    // waiting for a previously requested rDNS resolution
                                    // update the corresponding waiting address data
                                    info_traffic
                                        .addresses_waiting_resolution
                                        .entry(address_to_lookup)
                                        .and_modify(|data_info| {
                                            data_info.add_packet(
                                                exchanged_bytes,
                                                new_info.traffic_direction,
                                            );
                                        });
                                }
                                (_, true) => {
                                    // rDNS already resolved
                                    // update the corresponding host's data info
                                    let host = info_traffic
                                        .addresses_resolved
                                        .get(&address_to_lookup)
                                        .unwrap()
                                        .1
                                        .clone();
                                    info_traffic.hosts.entry(host).and_modify(|data_info_host| {
                                        data_info_host.data_info.add_packet(
                                            exchanged_bytes,
                                            new_info.traffic_direction,
                                        );
                                    });
                                }
                            }

                            //increment the packet count for the sniffed app protocol
                            info_traffic
                                .app_protocols
                                .entry(protocols.application)
                                .and_modify(|data_info| {
                                    data_info
                                        .add_packet(exchanged_bytes, new_info.traffic_direction);
                                })
                                .or_insert(DataInfo::new_with_first_packet(
                                    exchanged_bytes,
                                    new_info.traffic_direction,
                                ));
                        }
                    }
                }
            }
        }
    }
}
