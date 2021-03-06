#![no_main]
#![no_std]

pub use nrf52833_hal as hal;
pub use nrf52833_pac as pac;

use defmt_rtt as _;
use hal::{
    gpio::{p0, Level},
    prelude::_embedded_hal_serial_Read,
    uarte::{self, UarteRx},
};
use pac::{UARTE0, UARTE1};
use panic_probe as _;
use rtic::app;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        serial0: UarteRx<UARTE0>,
        serial1: UarteRx<UARTE1>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let p = cx.device;
        let p0parts = p0::Parts::new(p.P0);

        // enable UARTE1 interrupt
        p.UARTE0.intenset.modify(|_, w| w.endrx().set_bit());

        static mut SERIAL0_TX_BUF: [u8; 1] = [0; 1];
        static mut SERIAL0_RX_BUF: [u8; 1] = [0; 1];
        let (_, serial0) = uarte::Uarte::new(
            p.UARTE0,
            uarte::Pins {
                txd: p0parts.p0_21.into_push_pull_output(Level::High).degrade(),
                rxd: p0parts.p0_02.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        )
        .split(unsafe { &mut SERIAL0_TX_BUF }, unsafe {
            &mut SERIAL0_RX_BUF
        })
        .expect("Could not split serial0");

        // on NRF* serial interrupts are only called after the first read
        rtic::pend(pac::Interrupt::UARTE0_UART0);

        // enable UARTE1 interrupt
        p.UARTE1.intenset.modify(|_, w| w.endrx().set_bit());

        static mut SERIAL1_TX_BUF: [u8; 1] = [0; 1];
        static mut SERIAL1_RX_BUF: [u8; 1] = [0; 1];
        let (_, serial1) = uarte::Uarte::new(
            p.UARTE1,
            uarte::Pins {
                txd: p0parts.p0_22.into_push_pull_output(Level::High).degrade(),
                rxd: p0parts.p0_03.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        )
        .split(unsafe { &mut SERIAL1_TX_BUF }, unsafe {
            &mut SERIAL1_RX_BUF
        })
        .expect("Could not split serial1");

        // on NRF* serial interrupts are only called after the first read
        rtic::pend(pac::Interrupt::UARTE1);

        init::LateResources { serial0, serial1 }
    }

    #[task(binds = UARTE0_UART0, resources = [serial0])]
    fn uarte0_interrupt(cx: uarte0_interrupt::Context) {
        defmt::println!("uarte0 interrupt");
        while let Ok(b) = cx.resources.serial0.read() {
            defmt::println!("Byte on serial0: {}", b)
        }
    }

    #[task(binds = UARTE1, resources = [serial1])]
    fn uarte1_interrupt(cx: uarte1_interrupt::Context) {
        defmt::println!("uarte1 interrupt");
        while let Ok(b) = cx.resources.serial1.read() {
            defmt::println!("Byte on serial1: {}", b)
        }
    }
};
