#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(alloc_error_handler)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::*;
use embassy::executor::{task, Executor};
use embassy::traits::flash::Flash as _;
use embassy::util::Forever;

use futures::pin_mut;
use nrf_softdevice::{Flash, Softdevice};

static EXECUTOR: Forever<Executor> = Forever::new();

#[task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[task]
async fn flash_task(sd: &'static Softdevice) {
    let mut f = Flash::take(sd);
    pin_mut!(f);

    info!("starting erase");
    unwrap!(f.as_mut().erase(0x80000).await);
    info!("erased!");

    info!("starting write");
    unwrap!(f.as_mut().write(0x80000, &[1, 2, 3, 4]).await);
    info!("write done!");
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let (sdp, _p) = take_peripherals();
    let sd = Softdevice::enable(sdp, &Default::default());

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(softdevice_task(sd)));
        unwrap!(spawner.spawn(flash_task(sd)));
    });
}
