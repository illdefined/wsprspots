#![feature(iterator_fold_self)]

use std::cmp::{Ordering, PartialEq, PartialOrd, Eq, Ord};
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::num::ParseIntError;
use std::str::FromStr;
use std::vec::Vec;

use chrono::{DateTime, Utc, NaiveDateTime};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use unicase::Ascii;

/// Arc degree
#[modtype::use_modtype]
type Deg = modtype::F<360u16>;

/// Frequency band
///
/// The discriminant values are based on the numerical identifiers typically encountered in the
/// WSPRnet spot database.
#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
enum Band {
	B2200m = -1,
	B630m = 0,
	B160m = 1,
	B80m = 3,
	B60m = 5,
	B40m = 7,
	B30m = 10,
	B20m = 14,
	B17m = 18,
	B15m = 21,
	B12m = 24,
	B10m = 28,
	B6m = 50,
	B4m = 70,
	B2m = 144,
	B70cm = 432,
	B23cm = 1296
}

impl fmt::Display for Band {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		let (value, unit) = match self {
			Band::B2200m => ("2200", "m"),
			Band::B630m => ("630", "m"),
			Band::B160m => ("160", "m"),
			Band::B80m => ("80", "m"),
			Band::B60m => ("60", "m"),
			Band::B40m => ("40", "m"),
			Band::B30m => ("30", "m"),
			Band::B20m => ("20", "m"),
			Band::B17m => ("17", "m"),
			Band::B15m => ("15", "m"),
			Band::B12m => ("12", "m"),
			Band::B10m => ("10", "m"),
			Band::B6m => ("6", "m"),
			Band::B4m => ("4", "m"),
			Band::B2m => ("2", "m"),
			Band::B70cm => ("70", "cm"),
			Band::B23cm => ("23", "cm")
		};

		if formatter.alternate() {
			// Pretty‐print frequency band
			write!(formatter, "{} {}", value, unit)
		} else {
			// Frequency band identifier for ADIF
			write!(formatter, "{}{}", value, unit)
		}
	}
}

/// Transmission power
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
struct Power(i8);

impl Power {
	/// Convert power to Watts
	fn to_watts(self) -> f64 {
		10f64.powf(self.0 as f64 / 10.0 - 3.0)
	}
}

impl FromStr for Power {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Power(i8::from_str(s)?))
	}
}

impl fmt::Display for Power {
	fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
		if fmtr.alternate() {
			// Pretty‐print transmission power in Watts
			fn round(num: f64, mult: f64) -> f64 {
				(num / mult).round() * mult
			}

			let watts = self.to_watts();

			if watts >= 1000.0 {
				write!(fmtr, "{:.1} kW", self.to_watts() / 1000.0)
			} else if watts >= 100.0 {
				write!(fmtr, "{:.0} W", round(self.to_watts(), 10.0))
			} else if watts >= 10.0 {
				write!(fmtr, "{:.0} W", round(self.to_watts(), 1.0))
			} else if watts >= 1.0 {
				write!(fmtr, "{:.1} W", self.to_watts())
			} else if watts >= 0.1 {
				write!(fmtr, "{:.0} mW", round(self.to_watts(), 0.01) * 1000.0)
			} else if watts >= 0.01 {
				write!(fmtr, "{:.0} mW", round(self.to_watts(), 0.001) * 1000.0)
			} else if watts >= 0.001 {
				write!(fmtr, "{:.1} mW", self.to_watts() * 1000.0)
			} else {
				write!(fmtr, "≤ 1 mW")
			}
		} else {
			// Raw dBm value
			write!(fmtr, "{}", self.0)
		}
	}
}

/// WSPR spot
#[derive(Debug)]
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
	frequency: f64,
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
	/// Frequency band
	band: Band,
	/// Reporter software version
	version: String,
	/// WSPRnet error code
	code: i8
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
		self.distance as f64 / self.power.to_watts() * ((self.snr as f64 + 36.0) / 36.0)
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
			id: itr.next().ok_or_else(|| invalid("Missing ID field"))?.parse()?,
			timestamp: itr.next().ok_or_else(|| invalid("Missing timestamp field"))?.parse()?,
			call_rx: itr.next().ok_or_else(|| invalid("Missing reporter call sign field"))?.parse()?,
			grid_rx: itr.next().ok_or_else(|| invalid("Missing reporter grid field"))?.parse()?,
			snr: itr.next().ok_or_else(|| invalid("Missing SNR field"))?.parse()?,
			frequency: itr.next().ok_or_else(|| invalid("Missing frequency field"))?.parse()?,
			call_tx: itr.next().ok_or_else(|| invalid("Missing transmitter call sign field"))?.parse()?,
			grid_tx: itr.next().ok_or_else(|| invalid("Missing transmitter grid field"))?.parse()?,
			power: itr.next().ok_or_else(|| invalid("Missing transmission power field"))?.parse()?,
			drift: itr.next().ok_or_else(|| invalid("Missing frequency drift field"))?.parse()?,
			distance: itr.next().ok_or_else(|| invalid("Missing distance field"))?.parse()?,
			azimuth: Deg(itr.next().ok_or_else(|| invalid("Missing azimuth field"))?.parse()?),
			band: Band::from_isize(itr.next()
				.ok_or_else(|| invalid("Missing band field"))?.parse()?)
				.ok_or_else(|| invalid("Invalid frequency band"))?,
			version: itr.next().ok_or_else(|| invalid("Missing version field"))?.to_string(),
			code: itr.next().ok_or_else(|| invalid("Missing code field"))?.parse()?
		})
	}
}

impl fmt::Display for Spot {
	fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
		if fmtr.alternate() {
			// Pretty‐print spot information
			write!(fmtr, "{}: {} → {} ({} → {}) @ {:#}, {:#}, SNR {} dB, drift {:+} Hz/s, distance {} km",
			       self.datetime().format("%Y-%m-%d %H:%M"), self.call_tx, self.call_rx, self.grid_tx, self.grid_rx,
			       self.band, self.power, self.snr, self.drift, self.distance)
		} else {
			// Generate ADIF record for spot
			fn field(fmtr: &mut fmt::Formatter, name: &str, args: fmt::Arguments) -> fmt::Result {
				let value = fmt::format(args);
				writeln!(fmtr, "<{}:{}>{}", name, value.len(), value)
			}

			let datetime = self.datetime();
			field(fmtr, "QSO_DATE", format_args!("{}", datetime.format("%Y%m%d")))?;
			field(fmtr, "TIME_ON", format_args!("{}", datetime.format("%H%M")))?;
			field(fmtr, "OPERATOR", format_args!("{}", self.call_rx))?;
			field(fmtr, "MY_GRIDSQUARE", format_args!("{}", self.grid_rx))?;
			field(fmtr, "RST_SENT", format_args!("{} dB", self.snr))?;
			field(fmtr, "FREQ", format_args!("{}", self.frequency))?;
			field(fmtr, "CALL", format_args!("{}", self.call_tx))?;
			field(fmtr, "GRIDSQUARE", format_args!("{}", self.grid_tx))?;
			field(fmtr, "RX_PWR", format_args!("{:.4}", self.power.to_watts()))?;
			field(fmtr, "DISTANCE", format_args!("{}", self.distance))?;
			field(fmtr, "BAND", format_args!("{}", self.band))?;
			field(fmtr, "MODE", format_args!("WSPR"))?;
			field(fmtr, "QSO_RANDOM", format_args!("Y"))?;
			field(fmtr, "SWL", format_args!("Y"))?;
			field(fmtr, "QSLMSG",
			      format_args!("WSPR spot on {:#} with {:#} ({} dBm), SNR {} dB, drift {:+} Hz/s, distance {} km",
			                   self.band, self.power, self.power, self.snr, self.drift, self.distance))?;
			field(fmtr, "COMMENT", format_args!("WSPRnet spot ID {}", self.id))?;
			field(fmtr, "NOTES", format_args!("SpotQ {:.0}", self.spotq()))?;
			writeln!(fmtr, "<EOR>")
		}
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
