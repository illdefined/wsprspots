#![feature(exclusive_range_pattern, half_open_range_patterns, hash_drain_filter)]

mod excluded;
use crate::excluded::EXCLUDED;

use std::cmp::{self, Ordering, PartialEq, PartialOrd, Eq, Ord};
use std::collections::{HashMap, HashSet, BTreeSet, VecDeque};
use std::convert::TryFrom;
use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;

use smartstring::alias::String;
use chrono::{DateTime, Utc, NaiveDateTime};
use unicase::Ascii;

/// Call sign
type Call = Ascii<String>;

/// Maidenhead locator
type Grid = Ascii<String>;

/// Frequency
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Frequency(u64);

impl Frequency {
	/// Get frequency in MHz
	fn mhz(&self) -> f64 {
		self.0 as f64 / 1e6
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
	call_rx: Call,
	/// Reporter Maidenhead locator
	grid_rx: Call,
	/// Signal‐to‐noise ratio in dB
	snr: i8,
	/// Frequency of the received signal in MHz
	frequency: Frequency,
	/// Transmitter call sign
	call_tx: Call,
	/// Transmitter Maidenhead locator
	grid_tx: Call,
	/// Transmission power as reported by the transmitting station in dBm
	power: Power,
	/// Frequency drift in Hz / s
	drift: i8,
	/// Approximate distance between transmitter and reporter along the great circle path in km
	distance: u16,
}

impl Spot {
	fn cycle(&self) -> u64 {
		self.timestamp / 120
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
		})
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

/// WSPR QSO
#[derive(Clone, Debug)]
struct Qso {
	/// Operator call sign
	call_op: Call,
	/// Contact call sign
	call_ct: Call,
	/// Operator Maidenhead locator
	grid_op: Grid,
	/// Contact Maidenhead locator
	grid_ct: Grid,
	/// Timestamp of start of QSO
	time_first: u64,
	/// Timestamp of end of QSO
	time_last: u64,
	/// Operator’s SNR
	snr_op: i8,
	/// Contact’s SNR
	snr_ct: i8,
	/// Operator’s transmit power
	power_op: Power,
	/// Contact’s transmit power
	power_ct: Power,
	/// Operator’s transmit frequency
	freq_op: Frequency,
	/// Contact’s transmit frequency
	freq_ct: Frequency,
	/// Operator’s frequency drift
	drift_op: i8,
	/// Contact’s frequency drift
	drift_ct: i8,
	/// Approximate distance between operator and contact along the great circle path in km
	distance: u16,
	/// Spot IDs
	spots: BTreeSet<u64>
}

impl Qso {
	fn new(op: &Spot, ct: &Spot) -> Self {
		let mut spots = BTreeSet::<u64>::new();
		spots.insert(op.id);
		spots.insert(ct.id);

		Qso {
			call_op: op.call_rx.clone(),
			call_ct: op.call_tx.clone(),
			grid_op: op.grid_rx.clone(),
			grid_ct: op.grid_tx.clone(),
			time_first: cmp::min(op.timestamp, ct.timestamp),
			time_last: cmp::max(op.timestamp, ct.timestamp),
			snr_op: ct.snr,
			snr_ct: op.snr,
			power_op: ct.power,
			power_ct: op.power,
			freq_op: ct.frequency,
			freq_ct: op.frequency,
			drift_op: ct.drift,
			drift_ct: op.drift,
			distance: op.distance,
			spots
		}
	}

	fn update(&mut self, op: &Spot, ct: &Spot) {
		self.time_first = cmp::min(self.time_first, cmp::min(op.timestamp, ct.timestamp));
		self.time_last = cmp::max(self.time_last, cmp::max(op.timestamp, ct.timestamp));
		self.snr_op = cmp::max(self.snr_op, ct.snr);
		self.snr_ct = cmp::max(self.snr_ct, op.snr);
		self.drift_op = cmp::max(self.drift_op, ct.drift);
		self.drift_ct = cmp::max(self.drift_ct, op.drift);
		self.power_op = cmp::min(self.power_op, ct.power);
		self.power_ct = cmp::min(self.power_ct, op.power);
		self.spots.insert(op.id);
		self.spots.insert(ct.id);
	}

	fn cycle_last(&self) -> u64 {
		self.time_last / 120
	}

	/// Create [DateTime] object from start timestamp
	fn datetime_on(&self) -> DateTime<Utc> {
		DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.time_first as i64, 0), Utc)
	}

	/// Create [DateTime] object from end timestamp
	fn datetime_off(&self) -> DateTime<Utc> {
		DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp((self.time_last + 120) as i64, 0), Utc)
	}
}

impl fmt::Display for Qso {
	fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
		macro_rules! adif {
			($name:tt, $($arg:tt)*) => {{
				let value = format!($($arg)*);
				write!(fmtr, "<{}:{}>{}", $name, value.len(), value)
			}}
		}

		fn fmt_spots(set: &BTreeSet<u64>) -> String {
			let mut iter = set.iter();
			let mut st = String::new();

			// Assume that every set contains at least two IDs
			st.push_str(&iter.next().unwrap().to_string());
			for id in iter {
				st.push_str(", ");
				st.push_str(&id.to_string());
			}

			st
		}

		adif!("QSO_DATE", "{}", self.datetime_on().format("%Y%m%d"))?;
		adif!("TIME_ON", "{}", self.datetime_on().format("%H%M"))?;
		adif!("QSO_DATE_OFF", "{}", self.datetime_off().format("%Y%m%d"))?;
		adif!("TIME_OFF", "{}", self.datetime_off().format("%H%M"))?;
		adif!("OPERATOR", "{}", self.call_op)?;
		adif!("CALL", "{}", self.call_ct)?;
		adif!("MY_GRIDSQUARE", "{}", self.grid_op)?;
		adif!("GRIDSQUARE", "{}", self.grid_ct)?;
		adif!("RST_RCVD", "{:+03}", self.snr_op)?;
		adif!("RST_SENT", "{:+03}", self.snr_ct)?;
		adif!("FREQ", "{:.6}", self.freq_op.mhz())?;
		adif!("RX_FREQ", "{:.6}", self.freq_ct.mhz())?;

		if let Ok(band) = Band::try_from(self.freq_op) {
			adif!("BAND", "{}{}", band.0, band.1)?;
		}

		if let Ok(band) = Band::try_from(self.freq_ct) {
			adif!("BAND_RX", "{}{}", band.0, band.1)?;
		}

		adif!("TX_PWR", "{:.4}", self.power_op.watts())?;
		adif!("RX_PWR", "{:.4}", self.power_ct.watts())?;
		adif!("DISTANCE", "{}", self.distance)?;

		let band_op = match Band::try_from(self.freq_op) {
			Ok(band) => band.to_string(),
			Err(_) => self.freq_op.to_string()
		};

		let band_ct = match Band::try_from(self.freq_ct) {
			Ok(band) => band.to_string(),
			Err(_) => self.freq_ct.to_string()
		};

		let band_str = if band_op == band_ct {
			band_op
		} else {
			format!("{} (RX {})", band_op, band_ct)
		};

		adif!("QSLMSG",
		      "2-way WSPR spot on {} with {} ({} dBm), SNR {} dB, drift {:+} Hz/s, distance {} km",
		      band_str, self.power_ct, self.power_ct.0, self.snr_ct, self.drift_ct, self.distance)?;
		adif!("COMMENT",
		      "2-way WSPR spot on {} with {} ({} dBm), SNR {} dB, drift {:+} Hz/s, distance {} km",
		      band_str, self.power_ct, self.power_ct.0, self.snr_ct, self.drift_ct, self.distance)?;

		adif!("NOTES", "WSPRnet spot IDs {}", fmt_spots(&self.spots))?;
		adif!("MODE", "WSPR")?;
		adif!("QSO_RANDOM", "Y")?;
		write!(fmtr, "<EOR>")
	}
}

#[derive(Hash, PartialEq, Eq)]
struct QsoKey(Call, Grid, Grid, Band, Band);

fn main() -> std::io::Result<()> {
	let call_op = Ascii::new(env::args().nth(1).expect("Missing operator call sign"));
	let stdin = io::stdin();

	let mut cycle = 0u64;

	// Look back queues
	let mut rx = VecDeque::<Spot>::new();
	let mut tx = VecDeque::<Spot>::new();

	// Active QSOs
	let mut qsos = HashMap::<QsoKey, Qso>::new();

	// Unique call signs
	let mut contacts = HashSet::new();

	// Number of individual QSOs
	let mut num_qsos = 0usize;

	let pkg_name = env!("CARGO_PKG_NAME");
	let pkg_version = env!("CARGO_PKG_VERSION");
	println!("Mutual WSPR spots for {}\n\
	         <ADIF_VER:5>3.1.1\
	         <CREATED_TIMESTAMP:15>{}\
	         <PROGRAMID:{}>{}\
	         <PROGRAMVERSION:{}>{}\
	         <EOH>",
	         call_op, Utc::now().format("%Y%m%d %H%M%S"), pkg_name.len(), pkg_name, pkg_version.len(), pkg_version);

	for line in stdin.lock().lines() {
		let row = line?;

		let last = match row.parse::<Spot>() {
			Ok(spot) => spot,
			Err(err) => {
				eprintln!("Failed to parse row: {}\n\n{}", err, row);
				continue;
			}
		};

		if last.call_rx != call_op && last.call_tx != call_op {
			continue;
		}

		// Start new cycle
		if last.cycle() > cycle {
			cycle = last.cycle();

			// Purge reporter spots
			rx.retain(|spot| {
				spot.cycle() >= cycle - 2
			});

			// Purge transmitter spots
			tx.retain(|spot| {
				spot.cycle() >= cycle - 2
			});
		}

		let band_last = match Band::try_from(last.frequency) {
			Ok(band) => band,
			Err(err) => {
				eprintln!("Unable to determine band for {}: {}", last.frequency, err);
				continue;
			}
		};

		// Spots as reporter
		if last.call_rx == call_op {
			if EXCLUDED.contains(last.call_tx.as_ref()) {
				continue;
			}

			for spot in &tx {
				if spot.call_rx == last.call_tx &&
				   spot.grid_rx == last.grid_tx &&
				   spot.grid_tx == last.grid_rx {
					let band_spot = Band::try_from(spot.frequency).unwrap();
					qsos.entry(QsoKey(last.call_tx.clone(), last.grid_rx.clone(), last.grid_tx.clone(), band_last.clone(), band_spot)).or_insert_with(|| {
						Qso::new(&last, spot)
					}).update(&last, spot);
				}
			}

			rx.push_back(last);
		// Spots as transmitter
		} else if last.call_tx == call_op {
			if EXCLUDED.contains(last.call_rx.as_ref()) {
				continue;
			}

			for spot in &rx {
				if spot.call_tx == last.call_rx &&
				   spot.grid_rx == last.grid_tx &&
				   spot.grid_tx == last.grid_rx {
					let band_spot = Band::try_from(spot.frequency).unwrap();
					qsos.entry(QsoKey(last.call_rx.clone(), last.grid_tx.clone(), last.grid_rx.clone(), band_spot, band_last.clone())).or_insert_with(|| {
						Qso::new(spot, &last)
					}).update(spot, &last);
				}
			}

			tx.push_back(last);
		}


		// Log QSOs with no more spots
		for (_, qso) in qsos.drain_filter(|_, qso| {
			qso.cycle_last() < cycle - 2
		}) {
			println!("{}", qso);
			contacts.insert(qso.call_ct);
			num_qsos += 1;
		}
	}

	eprintln!("Logged {} QSOs with {} unique call signs", num_qsos, contacts.len());
	Ok(())
}
