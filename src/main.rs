use binary_search::Direction;
use geph4client::{
    dispatch,
    config::{CONFIG, Opt},
    service,
};

#[cfg(not(windows))]
fn main() -> anyhow::Result<()> {
    let ((largest_low, _), _) = binary_search::binary_search((1, ()), (65536, ()), |lim| {
        if rlimit::utils::increase_nofile_limit(lim).unwrap_or_default() >= lim {
            Direction::Low(())
        } else {
            Direction::High(())
        }
    });
    let _ = rlimit::utils::increase_nofile_limit(largest_low);
    log::info!("** set fd limit to {} **", largest_low);

    dispatch()
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use std::ops::Deref;

    let ((largest_low, _), _) = binary_search::binary_search((1, ()), (65536, ()), |lim| {
        if rlimit::utils::increase_nofile_limit(lim).unwrap_or_default() >= lim {
            Direction::Low(())
        } else {
            Direction::High(())
        }
    });
    let _ = rlimit::utils::increase_nofile_limit(largest_low);
    log::info!("** set fd limit to {} **", largest_low);

    match CONFIG.deref() {
        Opt::Install(_) => service::install()?,
        _ => service::start()?,
    };

    Ok(())
}
