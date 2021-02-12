#![feature(exclusive_range_pattern, half_open_range_patterns, iterator_fold_self)]

use std::cmp::{Ordering, PartialEq, PartialOrd, Eq, Ord};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;
use std::vec::Vec;

use chrono::{DateTime, Utc, NaiveDateTime};

use unicase::Ascii;

/// Arc degree
#[modtype::use_modtype]
type Deg = modtype::F<360u16>;

/// Frequency
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Frequency(u64);

impl Frequency {
	/// Get frequency in MHz
	fn mhz(&self) -> f64 {
		self.0 as f64 / 10e6
	}

	fn from_mhz(mhz: f64) -> Self {
		Frequency((mhz * 1e6).round() as u64)
	}
}

impl Display for Frequency {
	fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
		if fmtr.alternate() {
			match self.0 {
				0..1_000
					=> write!(fmtr, "{} Hz", self.0),
				1_000..1_000_000
					=> write!(fmtr, "{} kHz", self.0 as f64 / 1e3),
				1_000_000..1_000_000_000
					=> write!(fmtr, "{} MHz", self.0 as f64 / 1e6),
				_
					=> write!(fmtr, "{} GHz", self.0 as f64 / 1e9)
			}
		} else {
			write!(fmtr, "{} Hz", self.0)
		}
	}
}

/// Frequency band
#[derive(Clone, Debug)]
struct Band(&'static str, &'static str);

impl TryFrom<Frequency> for Band {
	type Error = io::Error;

	fn try_from(freq: Frequency) -> Result<Self, Self::Error> {
		match freq.0 {
			135_700..=137_800
				=> Ok(Band("2200", "m")),
			160_000..=190_000
				=> Ok(Band("1750", "m")),
			472_000..=479_000
				=> Ok(Band("630", "m")),
			1_800_000..=2_000_000
				=> Ok(Band("160", "m")),
			3_500_000..=4_000_000
				=> Ok(Band("80", "m")),
			5_060_000..=5_450_500
				=> Ok(Band("60", "m")),
			7_000_000..=7_300_000
				=> Ok(Band("40", "m")),
			10_100_000..=10_150_000
				=> Ok(Band("30", "m")),
			14_000_000..=14_350_000
				=> Ok(Band("20", "m")),
			18_068_000..=18_168_000
				=> Ok(Band("17", "m")),
			21_000_000..=21_450_000
				=> Ok(Band("15", "m")),
			24_890_000..=24_990_000
				=> Ok(Band("12", "m")),
			28_000_000..=29_700_000
				=> Ok(Band("10", "m")),
			40_000_000..=45_000_000
				=> Ok(Band("8", "m")),
			50_000_000..=54_000_000
				=> Ok(Band("6", "m")),
			54_000_001..=69_900_000
				=> Ok(Band("5", "m")),
			70_000_000..=71_000_000
				=> Ok(Band("4", "m")),
			144_000_000..=148_000_000
				=> Ok(Band("2", "m")),
			219_000_000..=225_000_000
				=> Ok(Band("1.25", "m")),
			420_000_000..=450_000_000
				=> Ok(Band("70", "cm")),
			902_000_000..=928_000_000
				=> Ok(Band("33", "cm")),
			1_240_000_000..=1_300_000_000
				=> Ok(Band("23", "cm")),
			2_300_000_000..=2_450_000_000
				=> Ok(Band("13", "cm")),
			3_300_000_000..=3_500_000_000
				=> Ok(Band("9", "cm")),
			5_600_000_000..=5_925_000_000
				=> Ok(Band("6", "cm")),
			10_000_000_000..=10_500_000_000
				=> Ok(Band("1.25", "cm")),
			24_000_000_000..=24_250_000_000
				=> Ok(Band("6", "mm")),
			75_500_000_000..=81_000_000_000
				=> Ok(Band("4", "mm")),
			119_980_000_000..=120_020_000_000
				=> Ok(Band("2.5", "mm")),
			142_000_000_000..=149_000_000_000
				=> Ok(Band("2", "mm")),
			241_000_000_000..=250_000_000_000
				=> Ok(Band("1", "mm")),
			_
				=> Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown frequency band"))
		}
	}
}

impl fmt::Display for Band {
	fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
		write!(fmtr, "{} {}", self.0, self.1)
	}
}


/// Transmission power
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
struct Power(i8);

impl Power {
	/// Convert power to Watts
	fn watts(self) -> f64 {
		10f64.powf(self.0 as f64 / 10.0 - 3.0)
	}

	fn from_dbm(dbm: i8) -> Power {
		Power(dbm)
	}
}

impl fmt::Display for Power {
	fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
		fn round(num: f64, mul: f64) -> f64 {
			(num / mul).round() * mul
		}

		let watts = self.watts();

		#[allow(illegal_floating_point_literal_pattern)]
		match watts {
			..1e-6
				=> write!(fmtr, "{} nW", watts * 1e9),
			1e-6..1e-5
				=> write!(fmtr, "{:.1} µW", watts * 1e6),
			1e-5..1e-4
				=> write!(fmtr, "{:.0} µW", round(watts, 1e-6) * 1e6),
			1e-4..1e-3
				=> write!(fmtr, "{:.0} µW", round(watts, 1e-5) * 1e6),
			1e-3..1e-2
				=> write!(fmtr, "{:.1} mW", watts * 1e3),
			1e-2..1e-1
				=> write!(fmtr, "{:.0} mW", round(watts, 1e-3) * 1e3),
			1e-1..1e0
				=> write!(fmtr, "{:.0} mW", round(watts, 1e-2) * 1e3),
			1e0..1e1
				=> write!(fmtr, "{:.1} W", watts),
			1e1..1e2
				=> write!(fmtr, "{:.0} W", round(watts, 1e0)),
			1e2..1e3
				=> write!(fmtr, "{:.0} W", round(watts, 1e1)),
			1e3..
				=> write!(fmtr, "{:.1} kW", watts / 1e3),
			_ => unreachable!()
		}
	}
}

/// WSPR spot
#[derive(Clone, Debug)]
struct Spot {
	/// Unique integer identifying a spot at WSPRnet
	id: u64,
	/// The time of the spot in seconds since the Unix epoch
	timestamp: u64,
	/// Reporter call sign
	call_rx: Ascii<String>,
	/// Reporter Maidenhead locator
	grid_rx: Ascii<String>,
	/// Signal‐to‐noise ratio in dB
	snr: i8,
	/// Frequency of the received signal in MHz
	frequency: Frequency,
	/// Transmitter call sign
	call_tx: Ascii<String>,
	/// Transmitter Maidenhead locator
	grid_tx: Ascii<String>,
	/// Transmission power as reported by the transmitting station in dBm
	power: Power,
	/// Frequency drift in Hz / s
	drift: i8,
	/// Approximate distance between transmitter and reporter along the great circle path in km
	distance: u16,
	/// Approximate direction from transmitter to reporter along the great circle path in degrees
	azimuth: Deg,
}

impl Spot {
	/// Create [DateTime] object from spot timestamp
	fn datetime(&self) -> DateTime<Utc> {
		DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.timestamp as i64, 0), Utc)
	}

	/// Calculate SpotQ metric for spot
	///
	/// The results may differ slightly from the reference implementation by Phil VKJ77 Perite
	/// at <http://wspr.vk7jj.com/> because of slightly different rounding when calculating the
	/// transmission power in Watts from dBm.
	fn spotq(&self) -> f64 {
		self.distance as f64 / self.power.watts() * ((self.snr as f64 + 36.0) / 36.0)
	}
}

impl FromStr for Spot {
	type Err = Box<dyn Error>;

	/// Parse Spot from CSV
	fn from_str(row: &str) -> Result<Self, Box<dyn Error>> {
		fn invalid(err: &'static str) -> Box<io::Error> {
			Box::new(io::Error::new(io::ErrorKind::InvalidData, err))
		}

		let itr = &mut row.split(',');

		Ok(Spot {
			id: itr.next()
				.ok_or_else(|| invalid("Missing ID field"))?
				.parse()?,
			timestamp: itr.next()
				.ok_or_else(|| invalid("Missing timestamp field"))?
				.parse()?,
			call_rx: itr.next()
				.ok_or_else(|| invalid("Missing reporter call sign field"))?
				.parse()?,
			grid_rx: itr.next()
				.ok_or_else(|| invalid("Missing reporter grid field"))?
				.parse()?,
			snr: itr.next()
				.ok_or_else(|| invalid("Missing SNR field"))?
				.parse()?,
			frequency: Frequency::from_mhz(itr.next()
				.ok_or_else(|| invalid("Missing frequency field"))?
				.parse()?),
			call_tx: itr.next()
				.ok_or_else(|| invalid("Missing transmitter call sign field"))?
				.parse()?,
			grid_tx: itr.next()
				.ok_or_else(|| invalid("Missing transmitter grid field"))?
				.parse()?,
			power: Power::from_dbm(itr.next()
				.ok_or_else(|| invalid("Missing transmission power field"))?
				.parse()?),
			drift: itr.next()
				.ok_or_else(|| invalid("Missing frequency drift field"))?
				.parse()?,
			distance: itr.next()
				.ok_or_else(|| invalid("Missing distance field"))?
				.parse()?,
			azimuth: Deg(itr.next()
				.ok_or_else(|| invalid("Missing azimuth field"))?
				.parse()?)
		})
	}
}

impl fmt::Display for Spot {
	fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
		macro_rules! adif {
			($name:tt, $($arg:tt)*) => {{
				let value = format!($($arg)*);
				write!(fmtr, "<{}:{}>{}", $name, value.len(), value)
			}}
		}

		adif!("QSO_DATE", "{}", self.datetime().format("%Y%m%d"))?;
		adif!("TIME_ON", "{}", self.datetime().format("%H%M"))?;
		adif!("OPERATOR", "{}", self.call_rx)?;
		adif!("MY_GRIDSQUARE", "{}", self.grid_rx)?;
		adif!("RST_SENT", "{} dB", self.snr)?;
		adif!("FREQ_RX", "{:.6}", self.frequency.mhz())?;
		adif!("CALL", "{}", self.call_tx)?;
		adif!("GRIDSQUARE", "{}", self.grid_tx)?;
		adif!("RX_PWR", "{:.4}", self.power.watts())?;
		adif!("DISTANCE", "{}", self.distance)?;

		match Band::try_from(self.frequency) {
			Ok(band) => adif!("BAND_RX", "{}{}", band.0, band.1)?,
			Err(_) => ()
		}

		adif!("MODE", "WSPR")?;
		adif!("QSO_RANDOM", "Y")?;
		adif!("SWL", "Y")?;

		let band_or_freq = match Band::try_from(self.frequency) {
			Ok(band) => band.to_string(),
			Err(_) => self.frequency.to_string()
		};

		adif!("QSLMSG",
		      "WSPR spot on {} with {} ({} dBm), SNR {} dB, drift {:+} Hz/s, distance {} km",
		      band_or_freq, self.power, self.power.0, self.snr, self.drift, self.distance)?;
		adif!("COMMENT", "WSPRnet spot ID {}", self.id)?;
		adif!("NOTES", "SpotQ {:.0}", self.spotq())?;
		write!(fmtr, "<EOR>")
	}
}

impl PartialEq for Spot {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl PartialOrd for Spot {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.id.cmp(&other.id))
	}
}

impl Eq for Spot { }

impl Ord for Spot {
	fn cmp(&self, other: &Self) -> Ordering {
		self.id.cmp(&other.id)
	}
}

fn main() -> std::io::Result<()> {
	let call_op = Ascii::new(env::args().nth(1).expect("Missing operator call sign"));
	let stdin = io::stdin();

	// Collect all spots by transmitter
	let mut spots = BTreeMap::new();

	for line in stdin.lock().lines() {
		let row = line?;

		match row.split(',').nth(2) {
			Some(call_rx) if call_rx == call_op => {
				match row.parse::<Spot>() {
					Ok(spot) => {
						spots.entry(spot.call_tx.clone()).or_insert_with(Vec::new).push(spot);
					},
					Err(err) => {
						eprintln!("Failed to parse row: {}\n\n{}", err, row);
						continue;
					}
				}
			},
			Some(_) => { }
			None => {
				eprintln!("Unable to identify operator call sign in row:\n\n{}", row);
				continue;
			}
		}
	}

	let pkg_name = env!("CARGO_PKG_NAME");
	let pkg_version = env!("CARGO_PKG_VERSION");
	println!("WSPR spots for {}\n<ADIF_VER:5>3.1.1<CREATED_TIMESTAMP:15>{}<PROGRAMID:{}>{}<PROGRAMVERSION:{}>{}<EOH>",
	         call_op, Utc::now().format("%Y%m%d %H%M%S"), pkg_name.len(), pkg_name, pkg_version.len(), pkg_version);

	// Determine the best spot for every transmitter according to SpotQ
	for vec in spots.values() {
		match &vec.iter().fold_first(|best, spot| {
			if spot.spotq() > best.spotq() { spot } else { best }
		}) {
			Some(spot) => println!("{}", spot),
			None => continue
		};
	}

	Ok(())
}
