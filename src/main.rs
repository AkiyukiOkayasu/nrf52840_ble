#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use embassy_nrf as _; // time driver
use panic_probe as _; // panic handler

use core::mem;

use defmt::{info, *};
use embassy_executor::Spawner;
use embassy_nrf::interrupt::{Interrupt, InterruptExt, Priority};
use embassy_time::{Duration, Timer};
use futures::future::{select, Either};
use futures::pin_mut;
use nrf_softdevice::ble::advertisement_builder::{
    Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList, ServiceUuid16,
};
use nrf_softdevice::ble::{gatt_server, peripheral, Connection};
use nrf_softdevice::{raw, Softdevice};

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[nrf_softdevice::gatt_service(uuid = "180f")]
struct BatteryService {
    #[characteristic(uuid = "2a19", read, notify)]
    battery_level: u8,
}

#[nrf_softdevice::gatt_service(uuid = "9e7312e0-2354-11eb-9f10-fbc30a62cf38")]
struct FooService {
    #[characteristic(uuid = "9e7312e0-2354-11eb-9f10-fbc30a63cf38", read, notify)]
    foo: u16,
}

#[nrf_softdevice::gatt_server]
struct Server {
    bas: BatteryService,
    foo: FooService,
}

/// BatteryLevelのnotifyを1秒ごとに行う。
async fn notify_battery_level<'a>(server: &'a Server, connection: &'a Connection) {
    loop {
        let mut battery_level = server.bas.battery_level_get().unwrap();
        battery_level += 1;
        server.bas.battery_level_set(&battery_level).unwrap();
        info!("notify_BatteryLevel: {}", battery_level);

        match server.bas.battery_level_notify(connection, &battery_level) {
            Ok(_) => info!("Battery adc_raw_value: {=u8}", &battery_level),
            Err(err) => info!("Battery notification error: {:?}", err),
        };

        // Sleep for one second.
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    // SoftDeviceが割り込み優先度P0, P1, P4を使用するので、それ以外を使用する
    // GPIO, TimerはP2を使用する
    // その他必要な割り込みがある場合は、明示的に優先度を設定する。初期値はP0では動作しない。
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    // ペリフェラルの初期化
    let _p = embassy_nrf::init(config);

    // 割り込みの優先度をprintする
    for num in 0..48 {
        let interrupt = unsafe { mem::transmute::<u16, Interrupt>(num) };
        let is_enabled = InterruptExt::is_enabled(interrupt);
        let priority = InterruptExt::get_priority(interrupt);

        info!(
            "Interrupt {}: Enabled = {}, Priority = {}",
            num, is_enabled, priority
        );
    }

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 3,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    let server = unwrap!(Server::new(sd));
    unwrap!(spawner.spawn(softdevice_task(sd)));

    static ADV_DATA: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(ServiceList::Complete, &[ServiceUuid16::BATTERY])
        .full_name("HelloRust")
        .build();

    static SCAN_DATA: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .services_128(
            ServiceList::Complete,
            &[0x9e7312e0_2354_11eb_9f10_fbc30a62cf38_u128.to_le_bytes()],
        )
        .build();

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data: &ADV_DATA,
            scan_data: &SCAN_DATA,
        };
        let conn = unwrap!(peripheral::advertise_connectable(sd, adv, &config).await);

        info!("advertising done!");

        // Set the battery level to 12%.
        server.bas.battery_level_set(&12u8).unwrap();

        let notify_battery_level_future = notify_battery_level(&server, &conn);

        // Run the GATT server on the connection. This returns when the connection gets disconnected.
        //
        // Event enums (ServerEvent's) are generated by nrf_softdevice::gatt_server
        // proc macro when applied to the Server struct above
        let gatt_future = gatt_server::run(&conn, &server, |event| match event {
            ServerEvent::Bas(e) => match e {
                BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
                    info!("battery notifications: {}", notifications)
                }
            },
            ServerEvent::Foo(e) => match e {
                FooServiceEvent::FooCccdWrite { notifications } => {
                    info!("foo notifications: {}", notifications)
                }
            },
        });

        pin_mut!(notify_battery_level_future);
        pin_mut!(gatt_future);

        let _ = match select(notify_battery_level_future, gatt_future).await {
            Either::Left((_, _)) => {
                info!("notify")
            }
            Either::Right((_, _)) => {
                info!("gatt")
            }
        };
    }
}
